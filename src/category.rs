use std::path::Path;

use super::icon::IconData;

pub fn split_up_path(path: &Path) -> Vec<String> {
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

pub fn categorize_icon(icon: &IconData) -> Vec<String> {
    if let Some(path) = &icon.path {
        let mut categories = get_categories_from_path(path);

        categories.retain(|c| !c.contains(icon.name.as_str()));
        get_categories_from_path(path)
    } else {
        vec![]
    }
}

pub fn get_categories_from_path(path: &Path) -> Vec<String> {
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
    use std::path::PathBuf;

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
            path: Some(PathBuf::from("/usr/share/icons/Adwaita")),
            ..Default::default()
        };

        assert_eq!(
            categorize_icon(&icon),
            vec!["Adwaita"]
        );
    }
}
