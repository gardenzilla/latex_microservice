use std::{
  collections::HashMap,
  fs::{self, File},
  io::Write,
  process::Command,
};

use tempdir::TempDir;
use tokio::task::spawn_blocking;

const MAIN_TEX_NAME: &'static str = "main.tex";
const PDF_NAME: &'static str = "main.pdf";

pub struct LatexObject {
  main_latex_file: Option<Vec<u8>>,
  related_files: Option<HashMap<String, Vec<u8>>>,
}

impl LatexObject {
  pub fn new(main_latex_file: Vec<u8>, related_files: HashMap<String, Vec<u8>>) -> Self {
    Self {
      main_latex_file: Some(main_latex_file),
      related_files: Some(related_files),
    }
  }
  pub async fn to_pdf(&mut self) -> Result<Vec<u8>, String> {
    // Move out main latex content
    let main_data = self
      .main_latex_file
      .take()
      .ok_or("Nincs main latex content!".to_string())?;

    // Move out related files content
    let related_data = self
      .related_files
      .take()
      .ok_or("Nincs related files!".to_string())?;

    spawn_blocking(move || {
      // First create temp working dir
      let tmp = TempDir::new("latex_to_pdf_process").map_err(|e| e.to_string())?;

      // Create main latex file
      let main_latex_path = tmp.path().join(MAIN_TEX_NAME);
      let mut main_latex_file = File::create(main_latex_path).map_err(|e| e.to_string())?;

      // Write main latex content into file
      main_latex_file
        .write_all(main_data.as_slice())
        .map_err(|e| e.to_string())?;

      // Flush main file
      main_latex_file.flush().map_err(|e| e.to_string())?;

      // Create related files
      for file in related_data {
        // Create path
        let path = tmp.path().join(&file.0);
        // Create file
        let mut rfile = File::create(path).map_err(|e| e.to_string())?;
        // Write content to file
        rfile
          .write_all(file.1.as_slice())
          .map_err(|e| e.to_string())?;
        // Flush file
        rfile.flush().map_err(|e| e.to_string())?;
      }

      let mut cmd = Command::new("pdflatex");
      cmd.args(&[MAIN_TEX_NAME]);
      cmd.current_dir(tmp.path());
      let cmd_output = cmd.output().map_err(|e| e.to_string())?;

      if !cmd_output.status.success() {
        return Err(format!("Error while pdflatex render!"));
      }

      let pdf_bytes = fs::read(tmp.path().join(&PDF_NAME))
        .map_err(|e| format!("Failed to read the generated PDF file! {}", e))?;

      return Ok(pdf_bytes);
    })
    .await
    .map_err(|e| e.to_string())?
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_latex() {
    let latex = r#"
      \documentclass{article}
      \begin{document}
      First document. This is a simple example, with no 
      extra parameters or packages included.
      \end{document}
    "#;
    let mut tex_object = LatexObject::new(latex.as_bytes().to_owned(), HashMap::new());
    let result = tex_object.to_pdf().await;

    assert!(result.is_ok());
  }
}
