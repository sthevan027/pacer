//! De onde sai o token do GitHub: o do GitHub CLI (`gh auth token`), o mesmo que o `gh` usa nos
//! comandos dele. O Pacer só lê; não renova, não grava e não guarda o token em lugar nenhum.

use std::process::{Command, Stdio};

/// Função que devolve o token atual, ou `None` se não há login. Injetável para os testes não
/// dependerem do `gh` nem de credencial real.
pub type TokenSource = Box<dyn Fn() -> Option<String> + Send + Sync>;

#[cfg(windows)]
fn no_window(cmd: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW)
}

#[cfg(not(windows))]
fn no_window(cmd: &mut Command) -> &mut Command {
    cmd
}

/// Saída do `gh auth token` → token, ignorando vazio.
pub fn clean(stdout: &[u8]) -> Option<String> {
    let t = String::from_utf8_lossy(stdout).trim().to_string();
    (!t.is_empty()).then_some(t)
}

pub fn gh_token() -> Option<String> {
    let mut cmd = Command::new("gh");
    cmd.args(["auth", "token", "--hostname", "github.com"]).stdin(Stdio::null()).stderr(Stdio::null());
    let out = no_window(&mut cmd).output().ok()?;
    if !out.status.success() {
        return None;
    }
    clean(&out.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_the_output_and_drops_empty_ones() {
        assert_eq!(clean(b"gho_abc\r\n").as_deref(), Some("gho_abc"));
        assert_eq!(clean(b"  \n"), None);
        assert_eq!(clean(b""), None);
    }
}
