//! Help command

#[derive(Debug, Clone, Copy, Default)]
pub enum HelpTopic {
    #[default]
    General,
    Throw,
    Throws,
    Help,
    Quit,
}

impl HelpTopic {
    pub fn help(&self) -> &'static str {
        match self {
            HelpTopic::General => include_str!("general.txt"),
            HelpTopic::Throw => todo!(),
            HelpTopic::Throws => todo!(),
            HelpTopic::Help => todo!(),
            HelpTopic::Quit => todo!(),
        }
    }
}
