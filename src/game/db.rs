//! Use local db for now

use std::sync::OnceLock;

use tracing::warn;

use crate::game::model::Quiz;

const QUIZ_DB_DIR: &str = "data";
const QUIZ_DB_FILE: &str = "quiz.csv";

pub struct QuizDB {
    // TODO: use HashMap (category, Vec<Quiz>)
    quizzes: Vec<Quiz>,
}

impl QuizDB {
    pub fn get_random_quiz(&self) -> &Quiz {
        let index = rand::random::<usize>() % self.quizzes.len();
        &self.quizzes[index]
    }
}

pub fn quiz_db() -> &'static QuizDB {
    static INSTANCE: OnceLock<QuizDB> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        let current_dir = std::env::current_dir().unwrap();
        let quiz_db_file = current_dir.join(QUIZ_DB_DIR).join(QUIZ_DB_FILE);

        let mut reader = csv::Reader::from_path(quiz_db_file).unwrap();
        let mut quizzes = Vec::new();

        for result in reader.deserialize() {
            let record: Quiz = match result {
                Ok(record) => record,
                Err(e) => {
                    warn!("{:<12} - Quiz load failed: {}", "GAME_DB", e);
                    continue;
                }
            };
            quizzes.push(record);
        }

        assert!(!quizzes.is_empty());

        QuizDB { quizzes }
    })
}
