use gtk::{IconPaintable, glib, prelude::*, subclass::prelude::*};
use std::{cell::Ref, collections::HashSet, path::PathBuf};

#[derive(Debug, Default, Clone)]
pub struct IconData {
    pub name: String,
    pub aliases: Vec<String>,
    pub tags: Vec<String>,
    pub path: Option<PathBuf>,
    pub symlink_path: Option<PathBuf>,
    pub symbolic: bool,
    pub symlink: bool,
}

impl IconData {
    /// Adds a list of aliases to the existing aliases of the icon.
    /// This method ensures that all aliases are unique by using a [HashSet].
    pub fn add_aliases(&mut self, aliases: Vec<String>) {
        self.aliases.extend(aliases);
        let set: HashSet<_> = self.aliases.drain(..).collect();

        self.aliases.extend(set);
    }
}

mod imp {
    use std::{
        cell::{Cell, RefCell},
        path::Path,
    };

    use gtk::glib::Properties;

    use crate::icon_theme;

    use super::*;

    #[derive(Properties, Debug, Default)]
    #[properties(wrapper_type = super::IconObject)]
    pub struct IconObject {
        #[property(name = "name", get, set = set_name, member = name, type = String)]
        #[property(name = "aliases", get, set, member = aliases, type = Vec<String>)]
        #[property(name = "tags", get, member = tags, type = Vec<String>)]
        #[property(name = "symbolic", get, member = symbolic, type = bool)]
        #[property(name = "symlink", get, member = symlink, type = bool)]
        #[property(
            name = "path",
            get = |o: &Self| o.data.borrow().path.as_ref().map(|p| p.display().to_string()),
            type = Option<String>
        )]
        #[property(
            name = "symlink-path",
            get = |o: &Self| o.data.borrow().symlink_path.as_ref().map(|p| p.display().to_string()),
            type = Option<String>
        )]
        pub data: RefCell<IconData>,

        #[property(get, set)]
        pub paintable: RefCell<Option<IconPaintable>>,
        #[property(get, set = set_icon_size)]
        pub icon_size: Cell<u32>,
    }

    fn set_name(imp: &IconObject, name: &str) {
        imp.data.borrow_mut().name = name.to_string();
        imp.obj().notify_name();
        imp.render_icon(true);
    }

    fn set_icon_size(imp: &IconObject, icon_size: u32) {
        imp.icon_size.set(icon_size);
        imp.obj().notify_icon_size();
        imp.render_icon(false);
    }

    impl IconObject {
        pub fn init(&self) {
            if self.paintable.borrow().is_none() {
                self.render_icon(true);
            }
        }

        pub fn add_aliases(&self, aliases: Vec<String>) {
            self.data.borrow_mut().add_aliases(aliases);
            self.obj().notify_aliases();
        }

        /// Sets the data of the icon.
        /// This method will replace the current data with the new data.
        /// After the data is replaced, the icon will be re-rendered.
        pub fn set_data(&self, data: IconData, icon_size: u32) {
            self.data.borrow_mut().name = data.name.to_string();
            self.obj().notify_name();
            self.replace_data(data);
            self.obj().set_icon_size(icon_size);
        }

        fn replace_data(&self, data: IconData) {
            let mut current_data = self.data.borrow_mut();
            let mut notify = Vec::new();

            for (name, changed) in [
                ("aliases", current_data.aliases != data.aliases),
                ("tags", current_data.tags != data.tags),
                ("path", current_data.path != data.path),
                ("symbolic", current_data.symbolic != data.symbolic),
                ("symlink", current_data.symlink != data.symlink),
                (
                    "symlink-path",
                    current_data.symlink_path != data.symlink_path,
                ),
            ] {
                if changed {
                    notify.push(name);
                }
            }

            *current_data = data;

            drop(current_data);

            for n in notify {
                self.obj().notify(n);
            }
        }

        fn render_icon(&self, update_data: bool) {
            let mut data = self.data.borrow().clone();
            let size = self.icon_size.get();
            let paintable = icon_theme().lookup_icon(
                &data.name,
                &[],
                size as i32,
                1,
                gtk::TextDirection::Ltr,
                gtk::IconLookupFlags::empty(),
            );

            let outer = self.obj().clone();
            if update_data {
                if let Some(path) = paintable.file().and_then(|f| f.path()) {
                    let is_symlink = std::fs::symlink_metadata(&path)
                        .map(|m| m.file_type().is_symlink())
                        .unwrap_or(false);

                    if is_symlink {
                        let symlink_path = std::fs::read_link(&path).ok();
                        data.symlink_path = symlink_path;
                    } else {
                        data.symlink_path = None;
                    }

                    data.path = Some(path);
                    data.symlink = is_symlink;
                    data.tags = get_tags(&data);
                }

                data.symbolic = paintable.is_symbolic();

                self.replace_data(data);
            }

            outer.set_paintable(paintable);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IconObject {
        const NAME: &'static str = "NettIconViewerIcon";
        type Type = super::IconObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for IconObject {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    fn split_up_path(path: &Path) -> Vec<String> {
        path.iter()
            .filter_map(|s| {
                if s.to_str().unwrap() == "/" {
                    None
                } else {
                    Some(s.to_str().unwrap().to_string())
                }
            })
            .collect()
    }

    fn get_tags(icon: &IconData) -> Vec<String> {
        let now = std::time::Instant::now();
        log::trace!("Categorizing icon: \"{}\"", icon.name);
        if let Some(path) = &icon.path {
            log::trace!("Getting categories from: \"{}\"", path.display());
            let mut categories = get_tags_from_path(path);

            categories.retain(|c| !c.starts_with(icon.name.as_str()));

            log::trace!(
                "Categories retrieval took: {} µs",
                now.elapsed().as_micros()
            );

            categories
        } else {
            log::debug!("Icon has no path");
            vec![]
        }
    }

    fn get_tags_from_path(path: &Path) -> Vec<String> {
        let mut categories = split_up_path(path);

        categories.retain(|c| !["usr", "share", "icons", ".local", ".icons"].contains(&c.as_str()));

        if let Some((index, _)) = categories
            .iter()
            .enumerate()
            .find(|(_, c)| c.as_str() == "home")
        {
            if index == 0 {
                let user = categories[index + 1].clone();
                categories.remove(index);
                categories.retain(|c| c != &user);
            }
        }

        categories
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_split_up_path() {
            let path = PathBuf::from("/usr/share/icons/Adwaita");
            let result = split_up_path(&path);
            assert_eq!(result, vec!["usr", "share", "icons", "Adwaita"]);
        }

        #[test]
        fn test_categorize_icon() {
            let icon = IconData {
                name: "test".to_string(),
                path: Some(PathBuf::from("/usr/share/icons/Adwaita/test.svg")),
                ..Default::default()
            };

            assert_eq!(get_tags(&icon), vec!["Adwaita"]);
        }

        #[test]
        fn test_get_tags_from_path() {
            for (path, expected) in [
                (PathBuf::from("/usr/share/icons/Adwaita"), vec!["Adwaita"]),
                (PathBuf::from("/home/dev/.icons/Adwaita"), vec!["Adwaita"]),
                (
                    PathBuf::from("/home/dev/.local/share/icons/Adwaita"),
                    vec!["Adwaita"],
                ),
            ] {
                assert_eq!(get_tags_from_path(&path), expected);
            }
        }
    }
}

glib::wrapper! {
    pub struct IconObject(ObjectSubclass<imp::IconObject>);
}

impl IconObject {
    pub fn new(name: &str, icon_size: u32) -> Self {
        let icon: Self = glib::Object::builder()
            .property("name", name)
            .property("icon-size", icon_size)
            .build();

        icon
    }

    pub fn from_data(data: IconData, icon_size: u32) -> Self {
        let obj: Self = glib::Object::builder().build();
        obj.imp().set_data(data, icon_size);

        obj
    }

    pub fn add_aliases(&self, aliases: Vec<String>) {
        self.imp().add_aliases(aliases);
    }

    pub fn data(&self) -> Ref<IconData> {
        self.imp().data.borrow()
    }
}

impl From<IconObject> for IconData {
    fn from(icon: IconObject) -> Self {
        icon.data().clone()
    }
}
