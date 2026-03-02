#[derive(Debug)]
pub enum Tier {
    Deopt,
    Opt,
}

pub enum InterpreterResult {
    End,
    IoBreak,
    ToggleTier(Tier),
}
