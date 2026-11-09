//! Which file starts ZTBrowser again: the AppImage itself, never its temporary mount.

use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

use ztbrowser_core::launch::program_location::ProgramLocation;

struct RunningProgramScenario {
    appimage: Option<&'static str>,
}

impl RunningProgramScenario {
    const MOUNTED_EXECUTABLE: &'static str = "/tmp/.mount_InDuckX1/usr/bin/ztbrowser";

    fn given_started_from_appimage(appimage: &'static str) -> Self {
        Self { appimage: Some(appimage) }
    }

    fn given_installed_program() -> Self {
        Self { appimage: None }
    }

    fn when_locating(&self) -> PathBuf {
        ProgramLocation::choose(self.appimage.map(OsStr::new), PathBuf::from(Self::MOUNTED_EXECUTABLE))
    }
}

#[test]
fn should_start_the_appimage_file_instead_of_its_temporary_mount() {
    let program = RunningProgramScenario::given_started_from_appimage("/tmp/ZTBrowser-x86_64.AppImage").when_locating();

    assert_eq!(program, Path::new("/tmp/ZTBrowser-x86_64.AppImage"));
}

#[test]
fn should_start_the_running_executable_when_installed_as_a_package() {
    let program = RunningProgramScenario::given_installed_program().when_locating();

    assert_eq!(program, Path::new(RunningProgramScenario::MOUNTED_EXECUTABLE));
}

#[test]
fn should_ignore_an_appimage_variable_that_is_not_an_absolute_path() {
    let program = RunningProgramScenario::given_started_from_appimage("ZTBrowser.AppImage").when_locating();

    assert_eq!(program, Path::new(RunningProgramScenario::MOUNTED_EXECUTABLE));
}

struct AppImageProcessScenario {
    appimage: Option<&'static str>,
    appdir: Option<&'static str>,
    executable: &'static str,
}

impl AppImageProcessScenario {
    fn given_started_by_the_appimage_runtime() -> Self {
        Self {
            appimage: Some("/home/ola/Apps/ZTBrowser.AppImage"),
            appdir: Some("/tmp/.mount_InDuckX1"),
            executable: "/tmp/.mount_InDuckX1/usr/bin/ztbrowser",
        }
    }

    fn given_the_installed_program_in_a_shell_that_exports_appimage() -> Self {
        Self { appdir: None, executable: "/usr/bin/ztbrowser", ..Self::given_started_by_the_appimage_runtime() }
    }

    fn given_a_program_outside_the_image_although_both_variables_are_set() -> Self {
        Self { executable: "/home/ola/bin/ztbrowser", ..Self::given_started_by_the_appimage_runtime() }
    }

    fn when_asked_which_appimage_runs(&self) -> Option<PathBuf> {
        ProgramLocation::running_appimage(
            self.appimage.map(OsStr::new),
            self.appdir.map(OsStr::new),
            Path::new(self.executable),
        )
    }
}

#[test]
fn should_name_the_appimage_file_when_the_program_runs_from_its_mount() {
    let appimage = AppImageProcessScenario::given_started_by_the_appimage_runtime().when_asked_which_appimage_runs();

    assert_eq!(appimage, Some(PathBuf::from("/home/ola/Apps/ZTBrowser.AppImage")));
}

#[test]
fn should_not_take_an_installed_program_for_an_appimage_because_of_a_stray_variable() {
    let appimage = AppImageProcessScenario::given_the_installed_program_in_a_shell_that_exports_appimage()
        .when_asked_which_appimage_runs();

    assert_eq!(appimage, None);
}

#[test]
fn should_not_take_a_program_outside_the_mount_for_the_appimage() {
    let appimage = AppImageProcessScenario::given_a_program_outside_the_image_although_both_variables_are_set()
        .when_asked_which_appimage_runs();

    assert_eq!(appimage, None);
}
