use std::{fmt::Write, fs, path::PathBuf};

#[derive(Debug, Default)]
struct ClassInject {
    body: String,
    on_resume: String,
    on_pause: String,
    on_create: String,
}

#[derive(Debug, Default)]
struct Inject {
    imports: String,
    main_activity: ClassInject,
}

impl Inject {
    fn add(&mut self, other: Inject) {
        self.imports.push_str(&other.imports);
        self.main_activity.body.push_str(&other.main_activity.body);
        self.main_activity
            .on_resume
            .push_str(&other.main_activity.on_resume);
        self.main_activity
            .on_pause
            .push_str(&other.main_activity.on_pause);
        self.main_activity
            .on_create
            .push_str(&other.main_activity.on_create);
    }
}

fn parse_inject_template(file: &str) -> Inject {
    let mut res = Inject::default();
    let mut target = None;

    for line in file.lines() {
        if line.is_empty() {
            continue;
        }
        if line.starts_with("//%") && line.contains("IMPORTS") {
            assert!(target.is_none());

            target = Some(&mut res.imports);
            continue;
        }
        if line.starts_with("//%") && line.contains("MAIN_ACTIVITY_BODY") {
            assert!(target.is_none());

            target = Some(&mut res.main_activity.body);
            continue;
        }
        if line.starts_with("//%") && line.contains("MAIN_ACTIVITY_ON_CREATE") {
            assert!(target.is_none());

            target = Some(&mut res.main_activity.on_create);
            continue;
        }
        if line.starts_with("//%") && line.contains("MAIN_ACTIVITY_ON_RESUME") {
            assert!(target.is_none());

            target = Some(&mut res.main_activity.on_resume);
            continue;
        }
        if line.starts_with("//%") && line.contains("MAIN_ACTIVITY_ON_PAUSE") {
            assert!(target.is_none());

            target = Some(&mut res.main_activity.on_pause);
            continue;
        }
        if line.starts_with("//%") && line.contains("END") {
            assert!(target.is_some());
            target = None;
            continue;
        }
        if let Some(ref mut target) = target {
            writeln!(*target, "{}", line);
        }
    }
    res
}

#[test]
fn parse_inject_template0() {
    let file = r##"
// some comment
//

//% IMPORTS

import a.a.a;

import a.a.b;

//% END

//% MAIN_ACTIVITY_BODY

public int a;

//% END

//% MAIN_ACTIVITY_ON_CREATE

test();

//% END

"##;

    let injects = parse_inject_template(&file);
    assert_eq!(injects.imports, "import a.a.a;\nimport a.a.b;\n");
    assert_eq!(injects.main_activity.body, "public int a;\n");
    assert_eq!(injects.main_activity.on_create, "test();\n");
}

pub fn preprocess_main_activity(
    java_src: &str,
    package_name: &str,
    library_name: &str,
    inject_files: &[PathBuf],
) -> String {
    let res = java_src.replace("TARGET_PACKAGE_NAME", package_name);
    let res = res.replace("LIBRARY_NAME", &library_name);

    let mut inject = Inject::default();

    for file in inject_files {
        let src = fs::read_to_string(file).unwrap();
        inject.add(parse_inject_template(&src));
    }

    let m = &inject.main_activity;
    let res = res.replace("//% IMPORTS", &inject.imports);
    let res = res.replace("//% MAIN_ACTIVITY_BODY", &m.body);
    let res = res.replace("//% MAIN_ACTIVITY_ON_RESUME", &m.on_resume);
    let res = res.replace("//% MAIN_ACTIVITY_ON_PAUSE", &m.on_pause);
    let res = res.replace("//% MAIN_ACTIVITY_ON_CREATE", &m.on_create);

    res
}
