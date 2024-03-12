//! Use local db for now

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
use rand::Rng;
use phf::{phf_set, Set};

use tracing::warn;

use crate::game::model::Quiz;

const QUIZ_DB_DIR: &str = "data";
const QUIZ_DB_FILE: &str = "quiz.csv";

// category should be pre-defined (for now)
// TODO: csv 긁어올때 채우기? -> OnceLock?
// pub const QUIZ_CATEGORIES: HashSet<&'static str> = {
//     let mut set = HashSet::new();
//     set.insert("상식");
//     set.insert("넌센스");
//     set.insert("고사성어");
//     set
// };

// use phf
pub static QUIZ_CATEGORIES: Set<&'static str> = phf_set! {
    "상식",
    "넌센스",
    "고사성어",
};

pub struct QuizDB {
    quizzes: HashMap<String, Vec<Quiz>>,
}

impl QuizDB {
    pub fn get_any_random_quiz(&self) -> &Quiz {
        let keys: Vec<&String> = self.quizzes.keys().collect();
        let mut rng = rand::thread_rng();

        let key_index = rng.gen_range(0..keys.len());
        let random_key = keys[key_index];

        let quiz_vec = self.quizzes.get(random_key).unwrap();

        let quiz_index = rng.gen_range(0..quiz_vec.len());

        &quiz_vec[quiz_index]
    }

    pub fn get_random_quiz_by_category(&self, category: &String) -> Option<&Quiz> {
        let quizzes_by_category = self.quizzes.get(category)?;
        let index = rand::random::<usize>() % quizzes_by_category.len();
        Some(&quizzes_by_category[index])
    }
}

pub fn quiz_db() -> &'static QuizDB {
    static INSTANCE: OnceLock<QuizDB> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        let current_dir = std::env::current_dir().unwrap();
        let quiz_db_file = current_dir.join(QUIZ_DB_DIR).join(QUIZ_DB_FILE);

        let mut reader = csv::Reader::from_path(quiz_db_file)
            .expect("quiz db file not found");

        let mut quizzes: HashMap<String, Vec<Quiz>> = HashMap::new();

        for result in reader.deserialize() {
            let record: Quiz = match result {
                Ok(record) => record,
                Err(e) => {
                    warn!("{:<12} - Quiz load failed: {}", "GAME_DB", e);
                    continue;
                }
            };
            // quizzes.push(record);
            let category = record.category.clone();

            let quiz_vec = quizzes.entry(category).or_insert(Vec::new());
            quiz_vec.push(record);
        }

        assert!(!quizzes.is_empty());

        QuizDB { quizzes }
    })
}
