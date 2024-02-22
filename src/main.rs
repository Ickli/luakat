// #![windows_subsystem = "windows"] // <-- with this, user will not see output of his program.

use mlua::Lua;
use std::ops::DerefMut;
use fltk::{enums::*, prelude::*, utils::oncelock::*, *};
use std::path::{Path, PathBuf};

// Application and lua
static mut APP: Lazy<LuaVarsShowApp> = Lazy::new(|| LuaVarsShowApp::new(500, 500, 700,500,100,50,app::Scheme::Base, "Переменные"));
static mut LUA: Lazy<Lua> = Lazy::new(|| Lua::new());

// Variable window
const NEW_CELL_NAME_INPUT_ID: &str = "NEW_CELL_NAME_INPUT_ID";

// Input window
const INPWINDOW_INPUT_END: &str = "START";
const INPWINDOW_INPUT_PROMPT: &str = "";
static mut ERROR_ON_DISPLAY: bool = false;
const INPWINDOW_INPUT_ID: &str = "INPWINDOW_INPUT_ID";
const INPWINDOW_DISPLAY_ID: &str = "INPWINDOW_DISPLAY_ID"; // <-- Display's type is input::Input due to some odd bug with desired TextDisplay not showing at all
const STANDARD_PANIC_MESSAGE: &str = "<Error> Что-то пошло не так! <Error>\n- Попробуй исправить ошибку!";

// Images' names
const BACKGROUND_IMAGE_NAME: &str = "den-programmista.jpg";

// Lua scripts' names
const GLOBALTABLE_LUA_FUNC_NAME: &str = "global_table_additional_func.lua";

// Common paths
static IMAGES_PATH: Lazy<PathBuf> = Lazy::new(|| [std::env::current_dir().unwrap().to_str().unwrap(), "images"].iter().collect());
static LUA_SCRIPTS_PATH: Lazy<PathBuf> = Lazy::new(|| [std::env::current_dir().unwrap().to_str().unwrap(), "lua_scripts"].iter().collect());

// Specific paths
static BACKGROUND_IMAGE_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let mut pathbuf = IMAGES_PATH.clone();
    pathbuf.push(BACKGROUND_IMAGE_NAME);
    pathbuf
});
static GLOBALTABLE_LUA_FUNC_PATH: Lazy<PathBuf> =  Lazy::new(|| {
    let mut pathbuf = LUA_SCRIPTS_PATH.clone();
    pathbuf.push(GLOBALTABLE_LUA_FUNC_NAME);
    pathbuf
});


// Types of crucial inputs and other's
type INPWINDOW_DISPLAY_TYPE = input::MultilineInput;
type INPWINDOW_INPUT_TYPE = input::Input;

fn get_app_mut() -> &'static mut LuaVarsShowApp{
    unsafe {
        APP.deref_mut()
    }
}

fn get_lua() -> &'static Lua {
    unsafe {
        LUA.deref_mut()
    }
}

fn take_input() -> String {
    let mut widget = app::widget_from_id::<INPWINDOW_INPUT_TYPE>(INPWINDOW_INPUT_ID).unwrap();
    let mut input = widget.value();
    widget.set_value(INPWINDOW_INPUT_PROMPT);
    return input;
}

fn execute_and_empty_display() {
    let mut disp = app::widget_from_id::<INPWINDOW_DISPLAY_TYPE>(INPWINDOW_DISPLAY_ID).unwrap();
    let disp_stacked_input = disp.value();
    disp.set_value("");

    match get_lua().load(&disp_stacked_input).exec() {
        Ok(_) => return,
        Err(err) => {
            unsafe {
                ERROR_ON_DISPLAY = true;
            }
            panic!("{}", err);
        },
    }
}

fn process_input() {
    let taken_input = take_input();

    // if taken_input == INPWINDOW_INPUT_END { // <-- instead of crucial word will be START button
    //     execute_and_empty_display();
    //     return;
    // }

    let mut disp = app::widget_from_id::<INPWINDOW_DISPLAY_TYPE>(INPWINDOW_DISPLAY_ID).unwrap();
    unsafe {
        if ERROR_ON_DISPLAY {
            disp.set_value("");
            ERROR_ON_DISPLAY = false;
        }
    }

    disp.insert(&taken_input).unwrap();
    disp.insert("\n").unwrap();
}

#[derive(Clone, Debug)]
pub struct LuaVarsShowApp{
    app: app::App, 
    wind: window::Window,
    cell_width: i32,
    cell_height: i32,
    next_cell_position: [i32; 2],
    input_names: std::collections::HashSet<&'static str>, 
}


impl LuaVarsShowApp {
    fn new(x: i32, y: i32, width: i32, height: i32, cell_width: i32, cell_height: i32, scheme: app::Scheme, label: &str) -> LuaVarsShowApp {
        return LuaVarsShowApp {
            app: app::App::default().with_scheme(scheme),
            wind: window::Window::default().with_size(width, height).with_pos(x, y).with_label(label),
            cell_width,
            cell_height,
            next_cell_position: [10, 10],
            input_names: std::collections::HashSet::<&'static str>::new()
        }
    }

    fn iterate_to_next_cp(&mut self) {
        if self.next_cell_position[0] + self.cell_width > self.wind.width() - self.cell_width - 10 {
            self.next_cell_position[0] = 10;
            self.next_cell_position[1] += 20 + self.cell_height;
        }
        else { 
            self.next_cell_position[0] += self.cell_width + 10; 
        }
    }

    fn create_callback_for_input(&self, label: &str) {
        let mut input = app::widget_from_id::<input::Input>(
            get_app_mut().input_names.get(label).unwrap()
        ).unwrap();
        let captured_label = get_app_mut().input_names.get(label).unwrap();
        input.set_callback(move |inp| {
            get_lua().globals().set(*captured_label, inp.value()).unwrap();
        });
    }

    fn create_new_cell(app: &mut LuaVarsShowApp, label: &str, contents: &str) {
        let static_label: &'static str = Box::leak(Box::new(String::from(label).to_owned()));
        let mut frame = frame::Frame::new(app.next_cell_position[0], app.next_cell_position[1], app.cell_width, 0, "");
        frame.set_label(&label);
        get_app_mut().input_names.insert(static_label);
        let mut inp = input::Input::default().with_pos(app.next_cell_position[0], app.next_cell_position[1] + 10).with_size(app.cell_width, app.cell_height).with_id(
            get_app_mut().input_names.get(label).unwrap()
        );
        inp.set_value(contents);

        // app.cells.insert(String::from(label), inp.as_widget_ptr());
        app.create_callback_for_input(label);

        app.iterate_to_next_cp();
    }

    fn end_wind(&self) {
        self.wind.end();
    }

    fn run(&mut self) {
        self.wind.show();
        self.app.run().unwrap();
    }

    fn change_contents(label: &str, new_contents: &str) {
        app::widget_from_id::<input::Input>(
            get_app_mut().input_names.get(label).unwrap()
        ).unwrap().set_value(new_contents);
    }

    fn add_create_btn(app: &mut LuaVarsShowApp, x: i32, y: i32, w: i32, h: i32, label: &str)  {
        get_app_mut().wind.begin();
        let mut btn = button::Button::new(x, y, w, h, "");
        btn.set_label(label);

        btn.set_callback(move |btn| {
            btn.parent().expect("a").deactivate();
            btn.parent().expect("No parent for button").begin();
            let mut i = app::widget_from_id::<input::Input>(NEW_CELL_NAME_INPUT_ID).unwrap();
            get_lua().load(&(i.value() + "= 'empty'")).exec().unwrap();
            i.set_value("");
            btn.parent().expect("reason").end();
            btn.parent().expect("No parent for button").activate();
        });
    }

    fn update_variables() {
        get_lua().load(r#"
            for k, v in pairs(_G_K) do
                fltk_change_contents(k, tostring(_G[k]))
            end
        "#).exec().unwrap();
    }
}

pub struct LuaInputWindow {
    wind: window::Window,
    display_height: i32,
    btn_width: i32,
}

impl LuaInputWindow {
    fn new(w: i32, h: i32, display_height: i32, btn_width: i32, lc: Color, label: &str) -> LuaInputWindow {
        let mut wind = window::Window::default()
            .with_size(w, h)
            .right_of(&get_app_mut().wind,10)
            .with_label(label);

        // wind.set_color(bc);
        wind.set_label_color(lc);
   
        return LuaInputWindow {
            wind,
            display_height,
            btn_width,
        }
    }

    fn show(&mut self) {
        self.wind.show();
    }

    fn hide(&mut self) {
        self.wind.hide();
    }

    fn end(&mut self) {
        self.wind.end();
    }

    fn add_new_input(&mut self, x: i32, y: i32, w: i32, h: i32, tc: Color, bc: Color) {
        self.wind.begin();
        let mut inp = input::Input::default()
            .with_size(w, h)
            .with_pos(x,y)
            .with_id(INPWINDOW_INPUT_ID);

        inp.set_color(bc);
        inp.set_text_color(tc);

        inp.set_callback(|inp| {
            process_input();
        });

        inp.set_trigger(CallbackTrigger::EnterKey);
    }

    fn add_new_display(&mut self, x: i32, y: i32, w: i32, h: i32, tc: Color, bc: Color) {
        self.wind.begin();
        let mut disp = input::MultilineInput::new(x, y, w, h, "").with_id(INPWINDOW_DISPLAY_ID);
        disp.set_text_color(tc);
        disp.set_readonly(true);
        disp.set_color(bc);
    }

    fn add_undo_button(&mut self, tc: Color) {
        self.wind.begin();
        let mut undo_btn = button::Button::new(
            self.wind.width() - self.btn_width, 
            self.display_height, 
            self.btn_width, 
            self.wind.height() - self.display_height,
            "undo"
        );

        undo_btn.set_label_color(tc);

        undo_btn.set_callback(move |btn| {
            match app::widget_from_id::<INPWINDOW_DISPLAY_TYPE>(INPWINDOW_DISPLAY_ID).unwrap().undo() {
                Ok(_) => return,
                Err(_) => return,
            }
        });
    }

    fn add_start_button(&mut self, tc: Color) {
        self.wind.begin();
        let mut start_btn = button::Button::new(
            self.wind.width() - self.btn_width * 2,
            self.display_height,
            self.btn_width,
            self.wind.height() - self.display_height,
            "start",
        );

        start_btn.set_label_color(tc);

        start_btn.set_callback(|_| {
            execute_and_empty_display();
            LuaVarsShowApp::update_variables();
        });
    }

    fn add_view_and_input(&mut self, tc: Color, bc: Color) {
        self.add_new_display(0, 0, self.wind.width(), self.display_height, tc, bc); // display's callback is none
        self.add_new_input( // input's callback is initialized in add_new_input
            0, 
            self.display_height, 
            self.wind.width() - self.btn_width * 2, 
            self.wind.height() - self.display_height, 
            tc, 
            bc
        );
        self.add_undo_button(Color::Black);
        self.add_start_button(Color::Black);
    }

}

fn main() {
    initialise_lua_to_rust_handles();

    let mut global_table_script_do = String::from_iter(["dofile('", GLOBALTABLE_LUA_FUNC_PATH.to_str().unwrap(),"')"]);
    global_table_script_do = global_table_script_do.replace("\\", "/");
    get_lua().load(
        &global_table_script_do
    ).exec().unwrap();

    // FLTK stuff
    get_app_mut().wind.begin();

    let mut back = image::SharedImage::load(BACKGROUND_IMAGE_PATH.to_str().unwrap().replace("\\", "/")).unwrap();
    back.scale(get_app_mut().wind.width(), get_app_mut().wind.height(), false, true);
    let mut frame = frame::Frame::default().with_size(get_app_mut().wind.width(),get_app_mut().wind.height());
    frame.set_image(Some(back));

    input::Input::default().with_size(100,30).with_pos(10, get_app_mut().wind.height() - 40).with_id(NEW_CELL_NAME_INPUT_ID);
    LuaVarsShowApp::add_create_btn(get_app_mut(), 120, get_app_mut().wind.height() - 40, 50, 30, "create");

    get_app_mut().end_wind();

    get_app_mut().wind.make_resizable(true);

    // input window (console-like)
    let mut inpWindow = LuaInputWindow::new(400, 300, 265, 50, Color::DarkMagenta, "Ввод");
    inpWindow.add_view_and_input(Color::White, Color::Black);
    inpWindow.end();
    inpWindow.show();



    get_app_mut().run();
}


fn print_test() {
    println!("test function called");
    get_lua().globals().set("a", 1).unwrap();
    get_lua().globals().set("a", 1).unwrap();
    get_lua().globals().get::<&str, i32>("a").unwrap();
}

fn test_lua() {
    get_lua().globals().set("test_function", get_lua().create_function(|_,()| {
        print_test();
        Ok(())
    }).unwrap()).unwrap();
}

fn initialise_lua_to_rust_handles() {
    get_lua().globals().set("fltk_create_new_cell", get_lua().create_function( |_, (var, value) : (String, String)| {
        get_app_mut().wind.begin();
        get_app_mut().wind.deactivate();
        LuaVarsShowApp::create_new_cell(get_app_mut(), &var, &value);
        get_app_mut().wind.activate();
        get_app_mut().wind.end();
        Ok(())
    }).unwrap()).unwrap();

    get_lua().globals().set("fltk_change_contents", get_lua().create_function(|_, (var, value) : (String, String)| {
        get_app_mut().wind.begin();
        get_app_mut().wind.deactivate();
        LuaVarsShowApp::change_contents(&var, &value);
        get_app_mut().wind.activate();
        get_app_mut().wind.end();
        Ok(())
    }).unwrap()).unwrap();
}
