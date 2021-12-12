use crate::*;
use serde::{de::DeserializeOwned, Serialize};
use ssh2::Session;
use std::io::{Error, Read, Write};
use std::path::Path;

/// Synchronizes a client `MtcList` with a server `MtcList` on on a remote server using a given `&ssh2::Session`.
/// The `server_path` should be a path to the saved `MtcList` on the server.
/// Setting `overwrite` to true will result in the `client_list` being synced with itself
/// with a copy of the list being sent to the server. If the server doesn't have a file yet then `overwrite` should be true.
pub fn sync_remote<T>(
    session: &Session,
    client_list: &mut MtcList<T>,
    server_path: &Path,
    overwrite: bool,
) -> Result<(), Error>
where
    T: MtcItem + Clone + DeserializeOwned + Serialize,
{
    if overwrite {
        client_list.sync_self();
        // TODO: Create client to server function and call it here!
        return upload_file(session, server_path, &serde_json::to_string(client_list)?);
    }

    let content = download_file(session, server_path)?;
    let mut server_list: MtcList<T> = serde_json::from_str(&content)?;

    client_list.sync(&mut server_list);
    upload_file(session, server_path, &serde_json::to_string(&server_list)?)
}

fn download_file(session: &Session, remote_file_path: &Path) -> Result<String, Error> {
    let (mut remote_file, _) = session.scp_recv(remote_file_path)?;
    let mut content = String::new();
    remote_file.read_to_string(&mut content)?;

    remote_file.send_eof()?;
    remote_file.wait_eof()?;
    remote_file.close()?;
    remote_file.wait_close()?;

    Ok(content)
}

fn upload_file(session: &Session, remote_file_path: &Path, content: &str) -> Result<(), Error> {
    let mut remote_file = session.scp_send(remote_file_path, 0o644, content.len() as u64, None)?;
    remote_file.write(content.as_bytes())?;

    remote_file.send_eof()?;
    remote_file.wait_eof()?;
    remote_file.close()?;
    remote_file.wait_close()?;

    Ok(())
}
