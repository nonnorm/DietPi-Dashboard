use std::path::Path;

use proto::frontend::{RenameAction, Signal as FrontendSignal, SignalAction, UploadAction};
use sysinfo::{Pid, Signal};
use tokio::fs;

use crate::client::BackendContext;

pub fn process_signal(mut ctx: BackendContext, action: SignalAction) {
    let sys = &mut ctx.system();

    let signal = match action.signal {
        FrontendSignal::Kill => Signal::Kill,
        FrontendSignal::Pause => Signal::Stop,
        FrontendSignal::Term => Signal::Term,
        FrontendSignal::Resume => Signal::Continue,
    };

    let Some(proc) = sys.system.process(Pid::from_u32(action.pid)) else {
        return;
    };

    proc.kill_with(signal);
}

pub async fn new_file(path: String) {
    let path = Path::new(&path);
    if !path.exists() {
        let _ = fs::write(path, []).await;
    }
}

pub async fn new_folder(path: String) {
    let _ = fs::create_dir(path).await;
}

pub async fn rename(action: RenameAction) {
    let _ = fs::rename(action.from, action.to).await;
}

pub async fn delete_file(path: String) {
    let _ = fs::remove_file(path).await;
}

pub async fn delete_folder(path: String) {
    let _ = fs::remove_dir_all(path).await;
}

pub async fn write(action: UploadAction) {
    let _ = fs::write(action.path, action.data).await;
}
