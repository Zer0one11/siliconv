use std::io::{Read, Seek, Write};

use siliconv_core::{
    action::{Action, PlayerButton, RestartType, TimePoint, TimedAction},
    error::ReplayError,
    format::Format,
    meta::Meta,
    replay::{Replay, ReplaySerializable},
    version::GameVersion,
};
use siliconv_macros::Meta;
use slc_oxide::{self as slc, InputData, v3::ActionType};

#[derive(Meta)]
pub struct SilicateMeta {
    #[meta(default = 240.0)]
    pub tps: f64,
    pub seed: u64,
}

struct Slc2Meta {
    seed: u64,
    reserved: [u8; 56],
}

impl slc::Meta for Slc2Meta {
    fn from_bytes(bytes: &[u8]) -> Self {
        let mut seed = [0; 8];
        seed.copy_from_slice(&bytes[0..8]);
        let seed = u64::from_le_bytes(seed);

        let mut reserved = [0; 56];
        reserved.copy_from_slice(&bytes[8..64]);
        Slc2Meta { seed, reserved }
    }

    fn size() -> u64 {
        64
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut bytes = vec![0; 64];
        bytes[0..8].copy_from_slice(&self.seed.to_le_bytes());
        bytes[8..64].copy_from_slice(&self.reserved);
        bytes.into_boxed_slice()
    }
}

pub struct SilicateReplay {
    inner: Replay,
}

impl SilicateReplay {
    const SLC3_HEADER: [u8; 8] = [b'S', b'L', b'C', b'3', b'R', b'P', b'L', b'Y'];
    const SLC2_HEADER: [u8; 4] = [b'S', b'I', b'L', b'L'];

    fn read_slc3<R: Read + Seek>(reader: &mut R) -> Result<Self, ReplayError> {
        use slc::v3 as slc;

        let original = slc::Replay::read(reader)
            .map_err(|e| ReplayError::ReadError(format!("failed to read slc3 replay: {e}")))?;

        let action_atom = original
            .atoms
            .atoms
            .iter()
            .find_map(|atom| {
                if let slc::atom::AtomVariant::Action(action_atom) = atom {
                    Some(action_atom)
                } else {
                    None
                }
            })
            .ok_or(ReplayError::ReadError(
                "missing action atom in slc3 replay".to_string(),
            ))?;

        let meta = SilicateMeta {
            tps: original.metadata.tps,
            seed: original.metadata.seed,
        };

        let actions = action_atom
            .actions
            .iter()
            .map(|a| TimedAction {
                time: TimePoint::Frame(a.frame),
                action: match a.action_type {
                    ActionType::Jump => Action::Player {
                        button: PlayerButton::Jump,
                        hold: a.holding,
                        player2: a.player2,
                    },
                    ActionType::Left => Action::Player {
                        button: PlayerButton::Left,
                        hold: a.holding,
                        player2: a.player2,
                    },
                    ActionType::Right => Action::Player {
                        button: PlayerButton::Right,
                        hold: a.holding,
                        player2: a.player2,
                    },
                    ActionType::Restart => Action::Restart {
                        restart_type: RestartType::Restart,
                        seed: Some(a.seed),
                    },
                    ActionType::RestartFull => Action::Restart {
                        restart_type: RestartType::RestartFull,
                        seed: Some(a.seed),
                    },
                    ActionType::Death => Action::Restart {
                        restart_type: RestartType::Death,
                        seed: Some(a.seed),
                    },
                    ActionType::TPS => Action::TPS { tps: a.tps },
                    ActionType::Reserved => Action::Empty,
                },
                position: None,
            })
            .collect();

        Ok(SilicateReplay {
            inner: Replay {
                meta: Box::new(meta),
                actions,
                format: Format::Slc3,
                game_version: GameVersion::new(22, 74),
            },
        })
    }

    fn write_slc3<W: Write>(&self, writer: &mut W) -> Result<(), ReplayError> {
        use slc::v3 as slc;

        let meta = SilicateMeta::from_fields(self.inner.meta.fields());

        let mut action_atom = slc::builtin::ActionAtom::new();
        let mut current_frame = 0;
        action_atom.actions = self
            .inner
            .actions
            .iter()
            .filter_map(|a| {
                let TimePoint::Frame(frame) = a.time else {
                    return None;
                };

                let a = match &a.action {
                    Action::Player {
                        button,
                        hold,
                        player2,
                    } => Some(slc::Action::player(
                        current_frame,
                        frame - current_frame,
                        match button {
                            PlayerButton::Jump => ActionType::Jump,
                            PlayerButton::Left => ActionType::Left,
                            PlayerButton::Right => ActionType::Right,
                        },
                        *hold,
                        *player2,
                    )),
                    Action::Restart { restart_type, seed } => Some(slc::Action::death(
                        current_frame,
                        frame - current_frame,
                        match restart_type {
                            RestartType::Restart => ActionType::Restart,
                            RestartType::RestartFull => ActionType::RestartFull,
                            RestartType::Death => ActionType::Death,
                        },
                        seed.unwrap_or(2137),
                    )),
                    Action::TPS { tps } => Some(slc::Action::tps_change(
                        current_frame,
                        frame - current_frame,
                        *tps,
                    )),
                    _ => None,
                };

                current_frame = frame;

                a
            })
            .collect();

        let mut replay = slc::Replay::new(slc::Metadata::new(meta.tps, meta.seed, 0));
        replay.add_atom(slc::atom::AtomVariant::Action(action_atom));

        replay
            .write(writer)
            .map_err(|e| ReplayError::WriteError(format!("failed to write slc3 replay: {e}")))
    }

    fn read_slc2<R: Read + Seek>(reader: &mut R) -> Result<Self, ReplayError> {
        let replay = slc::Replay::<Slc2Meta>::read(reader)
            .map_err(|e| ReplayError::ReadError(format!("failed to read slc2 replay: {e}")))?;

        let meta = SilicateMeta {
            tps: replay.tps,
            seed: replay.meta.seed,
        };

        let actions = replay
            .inputs
            .iter()
            .filter_map(|input| match &input.data {
                InputData::Skip => None,
                InputData::Player(p) => Some((
                    input.frame,
                    Action::Player {
                        button: match p.button {
                            1 => PlayerButton::Jump,
                            2 => PlayerButton::Left,
                            3 => PlayerButton::Right,
                            _ => unreachable!(),
                        },
                        hold: p.hold,
                        player2: p.player_2,
                    },
                )),
                InputData::Restart => Some((
                    input.frame,
                    Action::Restart {
                        restart_type: RestartType::Restart,
                        seed: None,
                    },
                )),
                InputData::RestartFull => Some((
                    input.frame,
                    Action::Restart {
                        restart_type: RestartType::RestartFull,
                        seed: None,
                    },
                )),
                InputData::Death => Some((
                    input.frame,
                    Action::Restart {
                        restart_type: RestartType::Death,
                        seed: None,
                    },
                )),
                InputData::TPS(tps) => Some((input.frame, Action::TPS { tps: *tps })),
            })
            .map(|(frame, action)| TimedAction {
                time: TimePoint::Frame(frame),
                action,
                position: None,
            })
            .collect();

        Ok(SilicateReplay {
            inner: Replay {
                meta: Box::new(meta),
                actions,
                format: Format::Slc2,
                game_version: GameVersion::new(22, 74),
            },
        })
    }

    fn read_slc1<R: Read + Seek>(reader: &mut R) -> Result<Self, ReplayError> {
        let mut tps = [0; 8];
        reader.read_exact(&mut tps)?;
        let tps = f64::from_le_bytes(tps);

        let mut length = [0; 4];
        reader.read_exact(&mut length)?;
        let length = u32::from_le_bytes(length);

        let mut actions: Vec<TimedAction> = Vec::with_capacity(length as usize);

        for _ in 0..length {
            let mut state = [0; 4];
            reader.read_exact(&mut state)?;
            let state = u32::from_le_bytes(state);

            let frame = u64::from(state >> 4);
            let player2 = (state & 0b1000) != 0;
            let button = match (state & 0b0110) >> 1 {
                1 => PlayerButton::Jump,
                2 => PlayerButton::Left,
                3 => PlayerButton::Right,
                _ => unreachable!(),
            };
            let hold = (state & 0b0001) != 0;

            actions.push(TimedAction {
                time: TimePoint::Frame(frame),
                action: Action::Player {
                    button,
                    hold,
                    player2,
                },
                position: None,
            });
        }

        let mut seed = [0; 8];
        let seed = match reader.read_exact(&mut seed) {
            Ok(()) => u64::from_le_bytes(seed),
            Err(_) => 2137,
        };

        let meta = SilicateMeta { tps, seed };

        Ok(SilicateReplay {
            inner: Replay {
                meta: Box::new(meta),
                format: Format::Slc1,
                actions,
                game_version: GameVersion {
                    major: 22,
                    minor: 60,
                },
            },
        })
    }
}

impl ReplaySerializable for SilicateReplay {
    fn new(replay: Replay) -> Self {
        SilicateReplay { inner: replay }
    }

    fn into_replay(self) -> Replay {
        self.inner
    }

    fn read<R: Read + Seek>(reader: &mut R) -> Result<Self, ReplayError>
    where
        Self: Sized,
    {
        tracing::debug!("using the Silicate replay format");

        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic)?;
        reader.seek(std::io::SeekFrom::Start(0))?;

        match magic {
            Self::SLC3_HEADER => Self::read_slc3(reader),
            _ if magic.starts_with(&Self::SLC2_HEADER) => Self::read_slc2(reader),
            _ => Self::read_slc1(reader),
        }
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<(), ReplayError> {
        tracing::debug!("using the Silicate replay format");

        self.write_slc3(writer)
    }
}
