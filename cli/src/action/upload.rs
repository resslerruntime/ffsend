use std::path::Path;
use std::sync::{Arc, Mutex};

use failure::{err_msg, Fail};
use ffsend_api::action::upload::Upload as ApiUpload;
use ffsend_api::reqwest::Client;

use cmd::cmd_upload::CmdUpload;
use error::ActionError;
use progress::ProgressBar;
use util::open_url;
#[cfg(feature = "clipboard")]
use util::{print_error, set_clipboard};

/// A file upload action.
pub struct Upload<'a> {
    cmd: &'a CmdUpload<'a>,
}

impl<'a> Upload<'a> {
    /// Construct a new upload action.
    pub fn new(cmd: &'a CmdUpload<'a>) -> Self {
        Self {
            cmd,
        }
    }

    /// Invoke the upload action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), ActionError> {
        // Get API parameters
        let path = Path::new(self.cmd.file()).to_path_buf();
        let host = self.cmd.host();

        // Create a reqwest client
        let client = Client::new();

        // Create a progress bar reporter
        let bar = Arc::new(Mutex::new(ProgressBar::new_upload()));

        // Execute an upload action
        let file = ApiUpload::new(host, path).invoke(&client, bar)
            .map_err(|err| ActionError::Upload(err))?;

        // Get the download URL, and report it in the console
        let url = file.download_url(true);
        println!("Download URL: {}", url);

        // Open the URL in the browser
        if self.cmd.open() {
            if let Err(err) = open_url(url.clone()) {
                print_error(
                    err.context("Failed to open the URL in the browser")
                );
            };
        }

        // Copy the URL in the user's clipboard
        #[cfg(feature = "clipboard")]
        {
            if self.cmd.copy() {
                if set_clipboard(url.as_str().to_owned()).is_err() {
                    print_error(
                        err_msg("Failed to copy the URL to the clipboard")
                            .compat()
                    );
                }
            }
        }

        Ok(())
    }
}