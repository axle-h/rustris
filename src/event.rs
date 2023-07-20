use crate::game::board::DestroyLines;
use crate::game::tetromino::Minos;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameEvent {
    Spawn {
        player: u32,
        minos: Minos,
    },
    Fall,
    Move,
    SoftDrop,
    HardDrop {
        player: u32,
        minos: Minos,
        dropped_rows: u32,
    },
    Rotate,
    Lock {
        player: u32,
        minos: Minos,
        hard_or_soft_dropped: bool
    },
    Destroy(DestroyLines),
    Destroyed {
        player: u32,
        lines: DestroyLines,
        send_garbage_lines: u32,
        level_up: bool,
    },
    Hold,
    Paused,
    UnPaused,
    GameOver { player: u32, condition: GameOverCondition },
    Victory,
    Quit,
    NextTheme,
    ReceivedGarbage {
        player: u32,
        lines: u32
    },
    ReceivedGarbageLine {
        player: u32,
        line: u32
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HighScoreEntryEvent {
    CursorRight,
    CursorLeft,
    ChangeChar,
    Finished,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum GameOverCondition {
    /// Top Out: An opponent’s Line Attacks force existing Blocks past the top of the Buffer Zone
    TopOut,
    /// Lock Out: The player locks a whole Tetrimino down above the Skyline
    LockOut,
    /// Block Out: One of the starting cells of the Next Tetrimino is blocked by an existing Block
    BlockOut,
}
