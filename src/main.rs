use std::fs;

use gtk4::{Align, Application, ApplicationWindow, Button, CheckButton, glib, Label, ListBox,
           ListBoxRow, ScrolledWindow};
use gtk4::glib::{Object, user_config_dir};
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, BoxExt, ButtonExt, Cast, CheckButtonExt,
                    EditableExt, GtkWindowExt, ListBoxRowExt, ListModelExtManual, WidgetExt};

const APP_ID: &str = "io.github.hellx2.TodoList";

static mut TODO_LIST: Option<ListBox> = None;

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    gtk4::init().unwrap();

    app.connect_activate(on_activate);
    app.connect_shutdown(serialize);

    attach_css();

    app.run()
}

fn on_activate(app: &Application) {
    let main_window = ApplicationWindow::builder()
        .title("Todo List")
        .application(app)
        .build();

    let main_box = gtk4::Box::builder().orientation(gtk4::Orientation::Vertical).build();
    main_window.set_child(Some(&main_box));

    unsafe {
        TODO_LIST = Some(ListBox::builder().css_classes(["todo_list"]).build());

        deserialize();

        let a = TODO_LIST.clone().unwrap().observe_children().iter()
                         .map(|x| x.unwrap()).collect::<Vec<Object>>();
        a.iter().map(|x| x.clone().unsafe_cast::<ListBoxRow>()
                          .child().unwrap().downcast::<gtk4::Box>().unwrap())
         .for_each(|x| {
             x.first_child().as_ref().unwrap().clone().unsafe_cast::<CheckButton>().connect_toggled(|_| {
                 serialize(&Application::default());
             });
         });

        let todo_scrollable = ScrolledWindow::builder()
            .child(TODO_LIST.as_ref().unwrap())
            .css_classes(["scrollable"])
            .min_content_height(400)
            .max_content_width(1080)
            .min_content_width(600)
            .max_content_height(720)
            .build();

        main_box.append(&todo_scrollable);
    }

    let entry = gtk4::Entry::builder().css_classes(["entry"]).build();
    main_box.append(&entry);

    let btns_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .css_classes(["btns_box"])
        .halign(Align::Center)
        .spacing(20)
        .build();

    let button = gtk4::Button::with_label("Add");
    btns_box.append(&button);

    button.connect_clicked(move |_| {
        let text = entry.text();
        if !text.is_empty() {
            unsafe {
                let a = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
                a.append(&CheckButton::new());
                a.append(&Label::new(Some(text.as_str())));

                TODO_LIST.as_ref().unwrap().append(&a);
            }
        }
    });

    let delete_button = Button::with_label("Delete");
    btns_box.append(&delete_button);

    delete_button.connect_clicked(|_| {
        unsafe {
            TODO_LIST.as_ref().unwrap().remove(&TODO_LIST.as_ref().unwrap().selected_row().unwrap());
        }
    });

    let clear_button = Button::with_label("Clear");
    btns_box.append(&clear_button);

    clear_button.connect_clicked(|_| {
        unsafe {
            TODO_LIST.as_ref().unwrap().remove_all();
        }
    });

    main_box.append(&btns_box);

    let quit_button = Button::builder()
        .label("Quit")
        .css_classes(["quit_button"])
        .build();
    // Append to main_box so that it's on its own line.
    main_box.append(&quit_button);

    quit_button.connect_clicked(|_| {
        serialize(&Application::default());
        std::process::exit(0);
    });

    main_window.set_child(Some(&main_box));
    main_window.present();
}

fn serialize(_: &Application) {
    unsafe {
        let a = TODO_LIST.clone().unwrap().observe_children().iter()
                         .map(|x| x.unwrap()).collect::<Vec<Object>>();
        let b = a.iter().map(|x| x.clone().unsafe_cast::<ListBoxRow>()
                                  .child().unwrap().downcast::<gtk4::Box>().unwrap())
                 .map(|x| (
                     x.first_child().unwrap().unsafe_cast::<CheckButton>().is_active(),
                     x.last_child().unwrap().downcast::<Label>().unwrap().text().to_string()
                 )).collect::<Vec<(bool, String)>>();

        let mut path = user_config_dir();
        path.push("hellx2/todo-list/data.json");

        let json = serde_json::to_string(&b).unwrap();
        fs::write(path, json).unwrap();
    }
}

fn deserialize() {
    let mut path = user_config_dir();
    path.push("hellx2/todo-list");

    let mut filename = path.clone();
    filename.push("data.json");

    if fs::metadata(&filename).is_err() {
        fs::create_dir_all(path).unwrap();
        fs::write(&filename, "[]").unwrap();
    }

    let json = fs::read_to_string(&filename).unwrap();
    let todo_list = serde_json::from_str::<Vec<(bool, String)>>(&json).unwrap();

    unsafe {
        todo_list.iter().map(|x| {
            let a = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(10)
                .css_classes(["todo_list_item"])
                .build();
            a.append(&CheckButton::builder().active(x.0).build());
            a.append(&Label::new(Some(x.1.as_str())));
            a
        }).collect::<Vec<gtk4::Box>>()
                 .iter().for_each(|x| TODO_LIST.as_ref().unwrap().append(x));
    }
}

fn attach_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_string(include_str!("style.css"));
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_USER,
    );
}