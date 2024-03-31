use shared::{errors::UnifiedError, git_data::GitAuth};

fn main() -> Result<(), UnifiedError> {
    // Create a sample GitAuth instance
    let git_auth = GitAuth {
        user: "Dj-Codeman".to_string(),
        repo: "artisan_ws".to_string(),
        branch: "main".to_string(),
<<<<<<< HEAD
        token: "***".to_string(),
=======
        token: "xxxxxxxxxxxxxxxxx".to_string(),
>>>>>>> 255c99b (Functional RC with tests)
    };

    // Specify the file path to save the GitAuth data
    let file_path = "/etc/artisan.cf";

    // Save the GitAuth data to the file
    git_auth.save(file_path)?;

    Ok(())
}