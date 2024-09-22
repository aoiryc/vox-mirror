#[macro_use]
extern crate tracing;

use std::{path::PathBuf, rc::Rc, thread};

use anyhow::Result;
use flume::Sender;

slint::include_modules!();

fn main() -> Result<()> {
    let (audio_bridge, audio_worker) = audio::bridged();
    let audio_bridge = Rc::new(audio_bridge);

    thread::spawn(audio_worker);

    let main_window = MainWindow::new()?;

    {
        let audio_bridge = audio_bridge.clone();
        main_window.on_start_recording(move || {
            audio_bridge.record(&[]).unwrap();
            RecorderState::Started
        });
    }
    main_window.on_stop_recording(move || RecorderState::Stopped);

    main_window.show()?;

    slint::run_event_loop()?;

    Ok(())
}

mod audio {
    use std::{any::Any, marker::PhantomData, result};

    use cpal::{
        traits::{DeviceTrait, HostTrait},
        Device, DevicesError, Host,
    };
    use thiserror::Error;

    pub fn bridged() -> (Bridged, Worker) {
        let (tx, rx) = flume::unbounded();
        let bridged = Bridged(tx);
        let worker: Worker = Box::new(|| {
            let mut rx = rx;
            let mut host = cpal::default_host();
            let mut input = host.default_input_device();
            let mut output = host.default_output_device();

            while let Ok((req, ret)) = rx.recv() {
                match req {
                    Request::GetInputDevices => todo!(),
                    Request::GetOutputDevices => todo!(),
                    Request::Playback => todo!(),
                    Request::Record => {
                        ret.send(Ok(Response::Record)).ok();
                    }
                }
            }
            Ok(())
        });
        (bridged, worker)
    }

    pub type Worker = Box<dyn FnOnce() -> Result<()> + Send + 'static>;

    pub struct Bridged(flume::Sender<(Request, oneshot::Sender<Result<Response>>)>);

    impl Bridged {
        pub fn get_input_devices(&self) -> Vec<String> {
            todo!()
        }
        pub fn get_output_devices(&self) -> Vec<String> {
            todo!()
        }
        pub fn playback(&self, data: &[f32]) -> Result<()> {
            todo!()
        }
        pub fn record(&self, data: &[f32]) -> Result<()> {
            let (tx, rx) = oneshot::channel();
            self.0.send((Request::Record, tx)).worker_stopped()?;
            let Response::Record = rx.recv().assert_always_respond()? else {
                unreachable!()
            };
            Ok(())
        }
    }

    enum Request {
        GetInputDevices,
        GetOutputDevices,
        Playback,
        Record,
    }

    enum Response {
        GetInputDevices(Vec<String>),
        GetOutputDevices(Vec<String>),
        Playback,
        Record,
    }

    #[derive(Debug, Error)]
    pub enum Error {
        #[error("audio device: {0}")]
        DevicesError(#[from] DevicesError),
        #[error("audio worker: stopped")]
        WorkerStopped,
    }

    pub type Result<T> = result::Result<T, Error>;

    trait ErrorExt<T>: IntoIterator<Item = T> + Sized {
        fn ok(self) -> Option<T> {
            self.into_iter().next()
        }
        fn worker_stopped(self) -> Result<T> {
            self.ok().ok_or(Error::WorkerStopped)
        }
        fn assert_always_respond(self) -> T {
            self.ok().unwrap()
        }
    }

    impl<T, E> ErrorExt<T> for result::Result<T, E> {}
}

struct Tape {
    name: String,
    index_end: u64,
    data: Vec<i32>,
}
