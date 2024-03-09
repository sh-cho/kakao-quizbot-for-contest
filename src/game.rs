pub enum Command {
    Start,
    Stop,
    Answer(String),
    Ranking,
}

impl Command {
    pub fn from_utterance(utterance: &str) -> Option<Command> {
        let utterance = utterance.trim();
        let command = utterance.splitn(2, ' ').next()?;

        match command {
            "시작" => Some(Command::Start),
            "중지" => Some(Command::Stop),
            // TODO: "정답" 명령어를 사용하지 않고, 바로 답 입력하도록
            "정답" => {
                let answer = utterance.splitn(2, ' ').nth(1)?;
                Some(Command::Answer(answer.to_string()))
            },
            "랭킹" => Some(Command::Ranking),
            _ => None,
        }
    }
}
