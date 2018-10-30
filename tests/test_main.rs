extern crate snapcall;
extern crate tempfile;

#[cfg(test)]
mod test {
    use snapcall::generate;
    use std::fs::File;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use tempfile::TempDir;

    #[test]
    fn test0001() {
        let work = TempDir::new().expect("TempDir::new() failed");

        let h_path = work.path().join("test0001-snapshot.h");
        let mut h = File::create(&h_path).expect(&format!("Could not create {:?}", &h_path));
        let c_path = test_path("test0001.c");
        match generate(&mut h, &c_path) {
            Ok(_) => {}
            Err(x) => panic!(x)
        }
        h.flush().expect("flush() failed");

        let main_path = work.path().join("main");
        let mut clang_main = Command::new("clang")
            .arg("-o").arg(&main_path)
            .arg("-I").arg(work.path())
            .arg("-Wall")
            .arg("-Werror")
            .arg(test_path("test0001-main.c"))
            .spawn()
            .expect("Could not start clang");
        assert!(clang_main.wait().expect("Could not wait for clang").success());
        let r_path = work.path().join("test0001-replay.h");
        let r = File::create(&r_path).expect(&format!("Could not create {:?}", &r_path));
        let mut main = Command::new(&main_path)
            .stdout(Stdio::from(r))
            .spawn()
            .expect("Could not start main");
        assert!(main.wait().expect("Could not wait for main").success());

        let replay_path = work.path().join("replay");
        let mut clang_replay = Command::new("clang")
            .arg("-o").arg(&replay_path)
            .arg("-I").arg(work.path())
            .arg("-Wall")
            .arg("-Werror")
            .arg(c_path)
            .arg(test_path("test0001-replay.c"))
            .spawn()
            .expect("Could not start clang");
        assert!(clang_replay.wait().expect("Could not wait for clang").success());

        let mut replay = Command::new(&replay_path)
            .spawn()
            .expect("Could not start replay");
        assert!(replay.wait().expect("Could not wait for replay").success());
    }

    fn test_path(name: &str) -> PathBuf {
        Path::new(file!())
            .parent().expect("file!().parent() failed")
            .join(name)
    }
}
