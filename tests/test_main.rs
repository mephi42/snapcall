extern crate snapcall;
extern crate tempfile;

#[cfg(test)]
mod test {
    use snapcall::generate;
    use std::fs::{File, OpenOptions};
    use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use tempfile::TempDir;

    #[test]
    fn test0001() {
        test("test0001");
    }

    #[test]
    fn test0002() {
        test("test0002");
    }

    #[test]
    fn test0003() {
        test("test0003");
    }

    fn cat(h: &mut File, h_path: &Path) {
        h.seek(SeekFrom::Start(0)).expect("seek() failed");
        let h_path_str = h_path.to_str().expect("to_str() failed");
        println!("--- BEGIN {} ---", &h_path_str);
        for line in BufReader::new(h).lines() {
            println!("{}", line.expect("lines() failed"));
        }
        println!("--- END {} ---", &h_path_str);
    }

    fn test(id: &str) {
        let work = TempDir::new().expect("TempDir::new() failed");

        let h_path = work.path().join(format!("{}-snapshot.h", id));
        let mut h = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&h_path)
            .expect(&format!("Could not create {:?}", &h_path));
        let c_path = test_path(&format!("{}.c", id));
        match generate(&mut h, &c_path) {
            Ok(_) => {}
            Err(x) => panic!(x)
        }
        h.flush().expect("flush() failed");
        cat(&mut h, &h_path);

        let main_path = work.path().join("main");
        let mut clang_main = Command::new("clang")
            .arg("-o").arg(&main_path)
            .arg("-I").arg(work.path())
            .arg("-Wall")
            .arg("-Werror")
            .arg(&c_path)
            .arg(test_path(&format!("{}-main.c", id)))
            .spawn()
            .expect("Could not start clang");
        assert!(clang_main.wait().expect("Could not wait for clang").success());
        let r_path = work.path().join(format!("{}-replay.h", id));
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
            .arg(&c_path)
            .arg(test_path(&format!("{}-replay.c", id)))
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
