use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Quiz {
    category: String,
    question: String,
    pub answer: String,
    comment: Option<String>,
}

impl Quiz {
    pub fn info_before(&self) -> String {
        format!("({}) Q. {}", self.category, self.question)
    }
    
    pub fn info_after(&self) -> String {
        format!("정답: {}\n{}", self.answer, self.comment.as_deref().unwrap_or_default())
    }
}
