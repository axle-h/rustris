use crate::config::APP_CONFIG_ROOT;
use serde::{Deserialize, Serialize};

const MAX_HIGH_SCORES: usize = 5;
const CONFIG_NAME: &str = "high_scores";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighScore {
    pub name: String,
    pub score: u32,
}

impl HighScore {
    pub fn new(name: &str, score: u32) -> Self {
        Self {
            name: name.to_string(),
            score,
        }
    }

    pub fn from_string(name: String, score: u32) -> Self {
        Self { name, score }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighScoreTable {
    scores: Vec<HighScore>,
}

impl Default for HighScoreTable {
    fn default() -> Self {
        Self {
            scores: vec![
                HighScore::new("ALEX", 500),
                HighScore::new("MOLLY", 400),
                HighScore::new("ESME", 300),
                HighScore::new("MOLLI", 200),
                HighScore::new("MOGS", 100),
            ],
        }
    }
}

impl HighScoreTable {
    pub fn load() -> Result<Self, String> {
        let mut result: Self =
            confy::load(APP_CONFIG_ROOT, CONFIG_NAME).map_err(|e| e.to_string())?;
        result.sorted();
        result.scores = result.scores.into_iter().take(MAX_HIGH_SCORES).collect();
        Ok(result)
    }

    pub fn save(&self) -> Result<(), String> {
        confy::store(APP_CONFIG_ROOT, CONFIG_NAME, self).map_err(|e| e.to_string())
    }

    pub fn entries(&self) -> &[HighScore] {
        self.scores.as_slice()
    }

    pub fn is_high_score(&self, new_score: u32) -> bool {
        self.try_get_score_index(new_score).is_some()
    }

    pub fn add_high_score(&mut self, new_score: HighScore) {
        let index = self
            .try_get_score_index(new_score.score)
            .expect("not a high score");
        self.scores.insert(index, new_score);
        if self.scores.len() > MAX_HIGH_SCORES {
            self.scores.pop();
        }
    }

    pub fn try_get_score_index(&self, new_score: u32) -> Option<usize> {
        match self
            .scores
            .iter()
            .enumerate()
            .find(|(_, s)| new_score > s.score)
            .map(|(i, _)| i)
        {
            None if self.scores.len() < MAX_HIGH_SCORES => Some(self.scores.len()),
            Some(i) => Some(i),
            _ => None,
        }
    }

    fn sorted(&mut self) {
        self.scores.sort_by(|x, y| y.score.cmp(&x.score));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new(scores: Vec<HighScore>) -> HighScoreTable {
        let mut result = HighScoreTable { scores };
        result.sorted();
        result
    }

    #[test]
    fn adds_score_to_empty_table() {
        let mut table = new(vec![]);
        assert!(table.is_high_score(0));
        table.add_high_score(HighScore::new("A", 0));
        assert_eq!(table.scores, vec![HighScore::new("A", 0)]);
    }

    #[test]
    fn adds_score_to_bottom() {
        let mut table = new(vec![HighScore::new("A", 1)]);
        assert!(table.is_high_score(0));
        table.add_high_score(HighScore::new("B", 0));
        assert_eq!(
            table.scores,
            vec![HighScore::new("A", 1), HighScore::new("B", 0)]
        );
    }

    #[test]
    fn adds_score_to_top() {
        let mut table = new(vec![HighScore::new("A", 0)]);
        assert!(table.is_high_score(1));
        table.add_high_score(HighScore::new("B", 1));
        assert_eq!(
            table.scores,
            vec![HighScore::new("B", 1), HighScore::new("A", 0)]
        );
    }

    #[test]
    fn not_a_high_score() {
        let table = new(vec![
            HighScore::new("A", 10),
            HighScore::new("B", 9),
            HighScore::new("C", 8),
            HighScore::new("D", 7),
            HighScore::new("E", 6),
        ]);
        assert!(!table.is_high_score(6));
    }

    #[test]
    fn inserts_new_high_score_in_middle() {
        let mut table = new(vec![
            HighScore::new("A", 10),
            HighScore::new("B", 9),
            HighScore::new("C", 8),
            HighScore::new("D", 7),
            HighScore::new("E", 6),
        ]);
        assert!(table.is_high_score(8));
        table.add_high_score(HighScore::new("new", 8));
        assert_eq!(
            table.scores,
            vec![
                HighScore::new("A", 10),
                HighScore::new("B", 9),
                HighScore::new("C", 8),
                HighScore::new("new", 8),
                HighScore::new("D", 7)
            ]
        );
    }

    #[test]
    fn inserts_new_high_score_at_top() {
        let mut table = new(vec![
            HighScore::new("A", 10),
            HighScore::new("B", 9),
            HighScore::new("C", 8),
            HighScore::new("D", 7),
            HighScore::new("E", 6),
        ]);
        assert!(table.is_high_score(11));
        table.add_high_score(HighScore::new("new", 11));
        assert_eq!(
            table.scores,
            vec![
                HighScore::new("new", 11),
                HighScore::new("A", 10),
                HighScore::new("B", 9),
                HighScore::new("C", 8),
                HighScore::new("D", 7)
            ]
        );
    }

    #[test]
    fn inserts_new_high_score_at_bottom() {
        let mut table = new(vec![
            HighScore::new("A", 10),
            HighScore::new("B", 9),
            HighScore::new("C", 8),
            HighScore::new("D", 7),
            HighScore::new("E", 6),
        ]);
        assert!(table.is_high_score(7));
        table.add_high_score(HighScore::new("new", 7));
        assert_eq!(
            table.scores,
            vec![
                HighScore::new("A", 10),
                HighScore::new("B", 9),
                HighScore::new("C", 8),
                HighScore::new("D", 7),
                HighScore::new("new", 7)
            ]
        );
    }
}
