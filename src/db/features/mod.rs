//! Feature repository module

pub mod models;
pub mod repo;

pub use models::Feature;
pub use repo::FeatureRepository;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_utils::tests::setup_test_db;

    #[test]
    fn test_insert_and_list() {
        let (_temp, db) = setup_test_db();
        let repo = db.features();

        let feature = Feature {
            id: None,
            category: "functional".to_string(),
            description: "Test feature".to_string(),
            steps: vec!["Step 1".to_string(), "Step 2".to_string()],
            passes: false,
            verification_command: Some("echo test".to_string()),
            last_error: None,
        };

        let id = repo.insert(&feature).unwrap();
        assert!(id > 0);

        let (passing, remaining) = repo.count().unwrap();
        assert_eq!(passing, 0);
        assert_eq!(remaining, 1);
    }

    #[test]
    fn test_mark_passing() {
        let (_temp, db) = setup_test_db();
        let repo = db.features();

        let feature = Feature {
            id: None,
            category: "functional".to_string(),
            description: "Test feature".to_string(),
            steps: vec![],
            passes: false,
            verification_command: None,
            last_error: None,
        };

        repo.insert(&feature).unwrap();

        let (passing, remaining) = repo.count().unwrap();
        assert_eq!(passing, 0);
        assert_eq!(remaining, 1);

        repo.mark_passing("Test feature").unwrap();

        let (passing, remaining) = repo.count().unwrap();
        assert_eq!(passing, 1);
        assert_eq!(remaining, 0);
    }

    #[test]
    fn test_count() {
        let (_temp, db) = setup_test_db();
        let repo = db.features();

        // Insert some features
        for i in 0..5 {
            let feature = Feature {
                id: None,
                category: "functional".to_string(),
                description: format!("Feature {}", i),
                steps: vec![],
                passes: i % 2 == 0, // 0, 2, 4 pass; 1, 3 fail
                verification_command: None,
                last_error: None,
            };
            repo.insert(&feature).unwrap();
        }

        let (passing, remaining) = repo.count().unwrap();
        assert_eq!(passing, 3);
        assert_eq!(remaining, 2);
    }
}
