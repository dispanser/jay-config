use battery::{self, units::ratio};
use jay_config::keyboard::syms::SYM_Sys_Req;
use uom::{
    fmt::DisplayStyle::*,
    si::f32::*,
    si::time::{minute, second},
    Conversion,
};

use sysinfo::{CpuExt, CpuRefreshKind, RefreshKind, System, SystemExt};
use {
    chrono::{format::StrftimeItems, Local},
    jay_config::{
        config,
        embedded::grab_input_device,
        exec::Command,
        get_workspace,
        input::capability::{CAP_KEYBOARD, CAP_POINTER},
        input::{get_seat, input_devices, on_new_input_device, InputDevice, Seat},
        keyboard::{
            mods::{Modifiers, ALT, CTRL, MOD1, MOD4, SHIFT},
            syms::{
                SYM_3270_PrintScreen, SYM_Print, SYM_Return, SYM_SunPrint_Screen, SYM_Super_L,
                SYM_b, SYM_bracketleft, SYM_bracketright, SYM_c, SYM_d, SYM_f, SYM_h, SYM_i, SYM_j,
                SYM_k, SYM_l, SYM_m, SYM_o, SYM_p, SYM_q, SYM_r, SYM_slash, SYM_t, SYM_u, SYM_v,
                SYM_x, SYM_y, SYM_1, SYM_2, SYM_3, SYM_4, SYM_5, SYM_6, SYM_F1, SYM_F10, SYM_F11,
                SYM_F12, SYM_F2, SYM_F3, SYM_F4, SYM_F5, SYM_F6, SYM_F7, SYM_F8, SYM_F9,
            },
        },
        quit, reload,
        status::set_status,
        switch_to_vt,
        timer::{duration_until_wall_clock_is_multiple_of, get_timer},
        video::on_graphics_initialized,
        Axis::{Horizontal, Vertical},
        Direction::{Down, Left, Right, Up},
    },
    std::{
        cell::{Cell, RefCell},
        time::Duration,
    },
};

const MOD: Modifiers = MOD4;

fn configure_seat(s: Seat) {
    s.bind(MOD | SYM_h, move || s.focus(Left));
    s.bind(MOD | SYM_j, move || s.focus(Down));
    s.bind(MOD | SYM_k, move || s.focus(Up));
    s.bind(MOD | SYM_l, move || s.focus(Right));

    s.bind(MOD | SHIFT | SYM_h, move || s.move_(Left));
    s.bind(MOD | SHIFT | SYM_j, move || s.move_(Down));
    s.bind(MOD | SHIFT | SYM_k, move || s.move_(Up));
    s.bind(MOD | SHIFT | SYM_l, move || s.move_(Right));

    s.bind(MOD | SYM_d, move || s.create_split(Horizontal));
    s.bind(MOD | SYM_v, move || s.create_split(Vertical));

    s.bind(MOD | SYM_t, move || s.toggle_split());
    s.bind(MOD | SYM_m, move || s.toggle_mono());
    s.bind(MOD | SYM_u, move || s.toggle_fullscreen());

    s.bind(MOD | SYM_f, move || s.focus_parent());

    s.bind(MOD | SHIFT | SYM_q, move || s.close());

    s.bind(MOD | SHIFT | SYM_f, move || s.toggle_floating());

    s.bind(MOD | SHIFT | SYM_Return, || Command::new("alacritty").spawn());

    s.bind(MOD | SYM_p, || Command::new("fuzzel").spawn());

    s.bind(MOD | SYM_bracketleft, || {
        Command::new("tessen").arg("-d").arg("fuzzel").spawn()
    });

    s.bind(MOD | SYM_bracketright, || {
        Command::new("/home/uncomfy/.config/river/book.nu").spawn()
    });

    s.bind(MOD | SYM_slash, || {
        Command::new("/home/uncomfy/.config/river/fnottctl_list.sh").spawn()
    });

    s.bind(MOD | SHIFT | SYM_i, || {
        Command::new("/home/uncomfy/.config/river/nubrowser.nu").spawn()
    });

    s.bind(MOD | SYM_i, || {
        Command::new("/home/uncomfy/.config/river/firefoxprofiles.nu").spawn()
    });

    // Screenshot will not work yet. Jay screenshot does :)

    s.bind(MOD | SHIFT | SYM_y, || {
        Command::new("yt-cli.nu").spawn()
    });

    s.bind(MOD | SYM_x, quit);

    s.bind(MOD | SHIFT | SYM_r, reload);

   let use_hc = Cell::new(true);
    s.bind(MOD | SHIFT | SYM_m, move || {
        let hc = !use_hc.get();
        use_hc.set(hc);
        log::info!("use hc = {}", hc);
        s.use_hardware_cursor(hc);
    });

    let fnkeys = [
        SYM_F1, SYM_F2, SYM_F3, SYM_F4, SYM_F5, SYM_F6, SYM_F7, SYM_F8, SYM_F9, SYM_F10, SYM_F11,
        SYM_F12,
    ];

    for (i, sym) in fnkeys.into_iter().enumerate() {
        s.bind(CTRL | ALT | sym, move || switch_to_vt(i as u32 + 1));
    }

    let numkeys = [SYM_1, SYM_2, SYM_3, SYM_4, SYM_5, SYM_6];
    for (i, sym) in numkeys.into_iter().enumerate() {
        let ws = get_workspace(&format!("{}", i + 1));
        s.bind(MOD | sym, move || s.show_workspace(ws));
        s.bind(MOD | SHIFT | sym, move || {
            s.set_workspace(ws);
            s.show_workspace(ws);
        });
    }

    fn do_grab(s: Seat, grab: bool) {
        for device in s.input_devices() {
            if device.has_capability(CAP_KEYBOARD) {
                log::info!(
                    "{}rabbing keyboard {:?}",
                    if grab { "G" } else { "Ung" },
                    device.0
                );
                grab_input_device(device, grab);
            }
        }
        if grab {
            s.unbind(SYM_y);
            s.bind(MOD | SYM_b, move || do_grab(s, false));
        } else {
            s.unbind(MOD | SYM_b);
            s.bind(SYM_y, move || do_grab(s, true));
        }
    }
    do_grab(s, false);
}

// fn check_battery() -> battery::Result<()> {
//     let manager = battery::Manager::new()?;
//     let mut battery = match manager.batteries()?.next() {
//         Some(Ok(battery)) => battery,
//         Some(Err(e)) => {
//             eprintln!("Unable to access battery information");
//             return Err(e);
//         }
//         None => {
//             eprintln!("Unable to find any batteries");
//             return Err(io::Error::from(io::ErrorKind::NotFound).into());
//         }
//     };
//     Ok(())
// }

fn setup_status() -> Result<(), battery::Error> {
    let time_format: Vec<_> = StrftimeItems::new("%Y-%m-%d %H:%M:%S").collect();
    let specifics = RefreshKind::new()
        .with_cpu(CpuRefreshKind::new().with_cpu_usage())
        .with_memory();
    let system = RefCell::new(System::new_with_specifics(specifics));
    let manager = battery::Manager::new()?;
    let update_status = move || {
        let batteries = manager.batteries().unwrap();
        let mut system = system.borrow_mut();
        system.refresh_specifics(specifics);
        let cpu_usage = system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / 100.0;
        let used = system.used_memory() as f64 / (1024 * 1024) as f64;
        let total = system.total_memory() as f64 / (1024 * 1024) as f64;
        let s = Time::format_args(minute, Abbreviation);
        // let battery_time_left = battery.time_to_empty;
        // let battery_percent_left = battery.energy();
        // let battery_percent = battery.energy_full();
        for mut battery in batteries {
            let status = format!(
                r##"BAT: M-{:?} DT-{:?} CT-{:?} Capacity-{:?}% <span color="#333333">|</span> MEM: {:.1}/{:.1} <span color="#333333">|</span> CPU: {:5.2} <span color="#333333">|</span> {}"##,
                battery.as_mut().expect("Battery not found").state(),
                s.with(
                    battery
                        .as_mut()
                        .expect("Battery not found")
                        .time_to_empty()
                        .unwrap_or_default()
                ),
                s.with(
                    battery
                        .as_mut()
                        .expect("Battery not found")
                        .time_to_full()
                        .unwrap_or_default()
                ),
                battery
                    .as_mut()
                    .expect("Battery not found")
                    .state_of_charge()
                    * Ratio::new::<battery::units::ratio::ratio>(100.0),
                used,
                total,
                cpu_usage,
                Local::now().format_with_items(time_format.iter())
            );
            set_status(&status);
        }
    };
    update_status();
    let period = Duration::from_secs(5);
    let timer = get_timer("status_timer");
    timer.repeated(duration_until_wall_clock_is_multiple_of(period), period);
    timer.on_tick(update_status);
    Ok(())
}

pub fn configure() {
    // Configure seats and input devices
    let seat = get_seat("default");
    configure_seat(seat);
    let handle_input_device = move |device: InputDevice| {
        if device.has_capability(CAP_POINTER) {
            device.set_left_handed(false);
            device.set_transform_matrix([[0.35, 0.0], [0.0, 0.35]]);
        }
        device.set_tap_enabled(true);
        device.set_seat(seat);
    };
    input_devices().into_iter().for_each(handle_input_device);
    on_new_input_device(handle_input_device);

    // Configure the status message
    setup_status().expect("Status not working");

    // Start programs
    on_graphics_initialized(|| {
        Command::new("systemctl")
            .arg("--user")
            .arg("import-environment")
            .arg("WAYLAND_DISPLAY")
            .arg("XDG_CURRENT_DESKTOP")
            .spawn();
        Command::new("dbus-update-activation-environment")
            .arg("SEATD_SOCK")
            .arg("DISPLAY")
            .arg("WAYLAND_DISPLAY")
            .arg("DESKTOP_SESSION=jay")
            .arg("XDG_CURRENT_DESKTOP=jay")
            .spawn();
        Command::new("/usr/libexec/polkit-kde-authentication-agent-1").spawn();
        Command::new("fnott").spawn();
        Command::new("wbg")
            .arg("/home/uncomfy/.config/river/backgrounds/romb.png")
            .spawn();
    });
}

config!(configure);
