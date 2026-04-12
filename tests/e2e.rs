#[path = "dsl.rs"]
mod dsl;

use dsl::TuiTest;

#[test]
#[ignore]
fn test_app_launches() {
    let mut tui = TuiTest::launch();
    tui.assert_contains("Omniknight");
    tui.assert_contains("Sessions");
    tui.quit();
}

#[test]
#[ignore]
fn test_create_workspace_and_terminal() {
    let mut tui = TuiTest::launch();

    tui.press("n");
    tui.settle(300);
    tui.assert_contains("New Workspace");

    tui.type_str("test-project");
    tui.press("Enter");
    tui.settle(1000);

    // Workspace created with auto-spawned shell, should see session tree
    tui.assert_contains("test-project");
    tui.assert_contains("shell");

    tui.quit();
}

#[test]
#[ignore]
fn test_session_list_navigation() {
    let mut tui = TuiTest::launch();

    // Start on SessionList pane
    tui.assert_contains("▸ Sessions");

    // l → Terminal pane
    tui.press("l");
    tui.settle(200);

    // h → back to SessionList
    tui.press("h");
    tui.settle(200);
    tui.assert_contains("▸ Sessions");

    tui.quit();
}

#[test]
#[ignore]
fn test_command_palette() {
    let mut tui = TuiTest::launch();

    tui.press(":");
    tui.assert_contains(">");

    tui.type_str("work");
    tui.settle(200);
    tui.assert_contains("Workspace");

    tui.press("Esc");
    tui.quit();
}

#[test]
#[ignore]
fn test_dialog_cancel() {
    let mut tui = TuiTest::launch();

    tui.press("n");
    tui.settle(300);
    tui.assert_contains("New Workspace");

    tui.press("Esc");
    tui.settle(200);
    tui.assert_contains("NORMAL");

    tui.quit();
}
