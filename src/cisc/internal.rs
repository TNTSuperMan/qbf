pub enum Tier {
    Deopt,
    Opt,
}

pub enum InterpreterResult {
    End,
    ToggleTier(Tier),
}
