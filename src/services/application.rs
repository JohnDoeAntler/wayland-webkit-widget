use gio::prelude::AppInfoExt;
use gtk::IconTheme;
use gtk::{prelude::IconThemeExt, IconLookupFlags};

pub fn get_applications() -> String {
    let mut ret = String::new();

    let k = IconTheme::new();

    gio::AppInfo::all().iter().for_each(|e| {
        ret.push_str(format!("app name: {:?}\n", e.name()).as_str());
        ret.push_str(format!("app display name: {:?}\n", e.display_name()).as_str());

        if let Some(description) = e.description() {
            ret.push_str(format!("app description: {:?}\n", description).as_str());
        }
        ret.push_str(format!("app path: {:?}\n", e.executable().to_str()).as_str());

        if let Some(i) = e.icon() {
            ret.push_str(format!("icon: {:?}\n", i).as_str());

            if let Some(name) = gio::prelude::IconExt::to_string(&i) {
                let info = k.lookup_icon(name.as_str(), 64, IconLookupFlags::FORCE_SYMBOLIC);

                if let Some(info) = info {
                    ret.push_str(format!("icon info: {:?}\n", info).as_str());
                    if let Some(filename) = info.filename() {
                        ret.push_str(format!("icon filename: {:?}\n", filename).as_str());
                    }
                }
            }
        }

        ret.push_str("\n");
    });

    ret
}
