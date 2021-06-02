use iui::prelude::*;
use iui::controls::{TabGroup, VerticalBox, HorizontalBox, Spacer, Label,
    Entry, LayoutStrategy, HorizontalSeparator, Button};
use crate::config::{self, Config};
use std::sync::{Arc, Mutex, mpsc::{channel, Sender}};
use std::thread;
use std::net::{IpAddr, SocketAddr, AddrParseError};
use std::str::FromStr;
use crate::hole_punch;

enum Connect {
    Client,
    Host,
}

pub fn run() {
    let ui = UI::init().unwrap();
    let mut win = Window::new(&ui, "Moly!", 250, 200, WindowType::HasMenubar);

    let config = Arc::new(Mutex::new(config::load_or_default()));

    let (sender, receiver) = channel();

    let mut tabs = TabGroup::new(&ui);
    let client_tab = connection_settings(&ui, config.clone(), true, sender.clone());
    let host_tab = connection_settings(&ui, config.clone(), false, sender);
    let server_tab = server_settings(&ui, config.clone());

    tabs.append(&ui, "Client", client_tab);
    tabs.append(&ui, "Host", host_tab);
    tabs.append(&ui, "Server Settings", server_tab);
    for page in 0..3 {
        tabs.set_margined(&ui, page, true);
    }

    win.set_margined(&ui, false);
    win.show(&ui);

    win.set_child(&ui, tabs);
    let mut stop = channel().0;
    thread::spawn(move || {
        loop {
            let action = receiver.recv().unwrap();
            let config = config.lock().unwrap();
            if let Ok(server_addr) = server_addr(&config.server_addr, config.server_port) {
                drop(stop.send(()));
                match action {
                    Connect::Client => {
                        if let Ok(sender) = hole_punch::client::start(
                            config.client_port,
                            config.client_name.to_owned(),
                            server_addr)
                        {
                            stop = sender;
                        }
                    },
                    Connect::Host => {
                        if let Ok(sender) = hole_punch::host::start(
                            config.host_port,
                            config.host_name.to_owned(),
                            server_addr)
                        {
                            stop = sender;
                        }
                    },
                }
            }
        }
    });

    ui.main();
}

fn connection_settings(ui: &UI, config: Arc<Mutex<Config>>, is_client: bool, sender: Sender<Connect>) -> VerticalBox {
    let mut host_name_box = HorizontalBox::new(ui);
    let mut host_entry = Entry::new(ui);
    let mut port_box = HorizontalBox::new(ui);
    let mut port_entry = Entry::new(ui);

    host_entry.on_changed(&ui, {
        let config = config.clone();
        move |name| {
            let mut config = config.lock().unwrap();
            if is_client {
                config.client_name = name;
            } else {
                config.host_name = name;
            }
        }
    });

    port_entry.on_changed(&ui, {
        let config = config.clone();
        move |port| {
            if let Ok(port) = port.parse() {
                let mut config = config.lock().unwrap();
                if is_client {
                    config.client_port = port;
                } else {
                    config.host_port = port;
                }
            }
        }
    });

    {
        let config = config.lock().unwrap();
        if is_client {
            host_entry.set_value(ui, &config.client_name);
            port_entry.set_value(ui, &format!("{}", config.client_port));
        } else {
            host_entry.set_value(ui, &config.host_name);
            port_entry.set_value(ui, &format!("{}", config.host_port));
        }
    }

    host_name_box.append(ui, Label::new(ui, "Name:"), LayoutStrategy::Compact);
    host_name_box.append(ui, Spacer::new(ui), LayoutStrategy::Compact);
    host_name_box.append(ui, host_entry, LayoutStrategy::Stretchy);
    host_name_box.set_padded(ui, true);

    port_box.append(ui, Label::new(ui, "Port:"), LayoutStrategy::Compact);
    port_box.append(ui, Spacer::new(ui), LayoutStrategy::Compact);
    port_box.append(ui, port_entry, LayoutStrategy::Stretchy);
    port_box.set_padded(ui, true);

    let mut connect_button = Button::new(ui, "Connect");
    connect_button.on_clicked(&ui, {
        move |_| {
            config::save(config.lock().unwrap().clone());
            if is_client {
                sender.send(Connect::Client).unwrap();
            } else {
                sender.send(Connect::Host).unwrap();
            }
        }
    });

    let mut vbox = VerticalBox::new(ui);
    vbox.append(ui, host_name_box, LayoutStrategy::Compact);
    vbox.append(ui, port_box, LayoutStrategy::Compact);
    vbox.append(ui, Spacer::new(ui), LayoutStrategy::Stretchy);
    vbox.append(ui, HorizontalSeparator::new(ui), LayoutStrategy::Compact);
    vbox.append(ui, Spacer::new(ui), LayoutStrategy::Compact);
    vbox.append(ui, connect_button, LayoutStrategy::Compact);
    vbox.set_padded(ui, true);
    vbox
}

fn server_settings(ui: &UI, config: Arc<Mutex<Config>>) -> VerticalBox {
    let mut addr_box = HorizontalBox::new(ui);
    let mut addr_entry = Entry::new(ui);
    let mut port_box = HorizontalBox::new(ui);
    let mut port_entry = Entry::new(ui);

    addr_entry.on_changed(&ui, {
        let config = config.clone();
        move |name| {
            let mut config = config.lock().unwrap();
            config.server_addr = name;
        }
    });

    port_entry.on_changed(&ui, {
        let config = config.clone();
        move |port| {
            if let Ok(port) = port.parse() {
                let mut config = config.lock().unwrap();
                config.server_port = port;
            }
        }
    });

    {
        let config = config.lock().unwrap();
        addr_entry.set_value(ui, &config.server_addr);
        port_entry.set_value(ui, &format!("{}", config.server_port));
    }

    addr_box.append(ui, Label::new(ui, "Address:"), LayoutStrategy::Compact);
    addr_box.append(ui, Spacer::new(ui), LayoutStrategy::Compact);
    addr_box.append(ui, addr_entry, LayoutStrategy::Stretchy);
    addr_box.set_padded(ui, true);

    port_box.append(ui, Label::new(ui, "Port:"), LayoutStrategy::Compact);
    port_box.append(ui, Spacer::new(ui), LayoutStrategy::Compact);
    port_box.append(ui, port_entry, LayoutStrategy::Stretchy);
    port_box.set_padded(ui, true);

    let mut save_button = Button::new(ui, "Save Settings");
    save_button.on_clicked(&ui, {
        move |_| {
            config::save(config.lock().unwrap().clone());
        }
    });

    let mut vbox = VerticalBox::new(ui);
    vbox.append(ui, addr_box, LayoutStrategy::Compact);
    vbox.append(ui, port_box, LayoutStrategy::Compact);
    vbox.append(ui, Spacer::new(ui), LayoutStrategy::Stretchy);
    vbox.append(ui, HorizontalSeparator::new(ui), LayoutStrategy::Compact);
    vbox.append(ui, Spacer::new(ui), LayoutStrategy::Compact);
    vbox.append(ui, save_button, LayoutStrategy::Compact);
    vbox.set_padded(ui, true);
    vbox
}

fn server_addr(addr: &str, port: u16) -> Result<SocketAddr, AddrParseError> {
    let ip = IpAddr::from_str(addr)?;
    Ok(SocketAddr::new(ip, port))
}
