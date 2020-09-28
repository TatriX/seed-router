#![allow(unused)]
use seed::Url;

fn main() {
    println!("Hello, world!");
}

#[derive(Debug, PartialEq)]
enum Routes {
    /// "/"
    Home,
    /// "/about"
    About,
    /// "/contacts" | "/contacts?show-map"
    Contacts { show_map: Option<bool> },
    /// "/manager/"
    Manager(manager::Routes),
}

mod manager {
    #[derive(Debug, PartialEq)]
    pub enum Routes {
        /// "/"
        Dashboard,
        /// "/users"
        Users(users::Routes),
    }

    pub mod users {
        #[derive(Debug, PartialEq)]
        pub enum Routes {
            /// "/"
            List,
            /// "/add"
            Add,
            /// "/edit/<id>?groups=all"
            Edit {
                id: usize,              // note this is part of the path
                groups: Option<String>, // Optional fields goes to "search"
            },
        }
    }
}

trait Router {
    fn from_url(url: Url) -> Option<Self>
    where
        Self: Sized;
    fn to_url(&self) -> Url;
}

impl Router for Routes {
    fn from_url(mut url: Url) -> Option<Self> {
        match url.next_path_part() {
            Some("about") => Some(Self::About),
            Some("contacts") => Some(Self::Contacts {
                show_map: url.search().get("show-map").map(|_| true), // TODO: actually parse value?
            }),
            Some("manager") => manager::Routes::from_url(url).map(Self::Manager),
            None => Some(Self::Home),
            Some(_) => None,
        }
    }

    fn to_url(&self) -> Url {
        match self {
            Routes::Home => Url::new(),
            Routes::About => Url::new().add_path_part("about"),
            Routes::Contacts { show_map } => {
                let mut url = Url::new().add_path_part("contacts");
                if let Some(true) = show_map {
                    url.search_mut().push_value("show-map", "".into());
                }
                url
            }
            Routes::Manager(routes) => prepend_path(vec!["manager"], routes.to_url()),
        }
    }
}

impl Router for manager::Routes {
    fn from_url(mut url: Url) -> Option<Self> {
        match url.next_path_part() {
            None => Some(Self::Dashboard),
            Some("users") => manager::users::Routes::from_url(url).map(Self::Users),
            Some(_) => None,
        }
    }

    fn to_url(&self) -> Url {
        match self {
            Self::Dashboard => Url::new(),
            Self::Users(routes) => prepend_path(vec!["users"], routes.to_url()),
        }
    }
}

impl Router for manager::users::Routes {
    fn from_url(mut url: Url) -> Option<Self> {
        match url.next_path_part() {
            None => Some(Self::List),
            Some("add") => Some(Self::Add),
            Some("edit") => url
                .next_path_part()
                .and_then(|id| id.parse().ok())
                .map(|id| Self::Edit {
                    id,
                    groups: url
                        .search()
                        .get("groups")
                        .and_then(|groups| groups.get(0).map(String::to_string)),
                }),
            Some(_) => None,
        }
    }

    fn to_url(&self) -> Url {
        match self {
            Self::List => Url::new(),
            Self::Add => Url::new().add_path_part("add"),
            Self::Edit { id, groups } => {
                let mut url = Url::new()
                    .add_path_part("edit")
                    .add_path_part(id.to_string());
                if let Some(groups) = groups {
                    url.search_mut().push_value("groups", groups.into());
                }
                url
            }
        }
    }
}

/// There is no easy way to prepend prefix in the current `Url`
/// implementation so we need to hack it here.
fn prepend_path(mut path: Vec<&str>, url: Url) -> Url {
    let mut path = path.clone(); // fix weird lifetime error
    for segment in url.path() {
        path.push(segment);
    }
    Url::new().set_path(path).set_search(url.search().clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_home() {
        let url: Url = "/".parse().unwrap();
        let route = Routes::from_url(url).unwrap();
        assert_eq!(route, Routes::Home);
        assert_eq!(route.to_url().to_string(), "/");
    }

    #[wasm_bindgen_test]
    fn test_404() {
        let url: Url = "/404".parse().unwrap();
        let route = Routes::from_url(url);
        assert_eq!(route, None);
    }

    #[wasm_bindgen_test]
    fn test_about() {
        let url: Url = "/about".parse().unwrap();
        let route = Routes::from_url(url).unwrap();
        assert_eq!(route, Routes::About);
        assert_eq!(route.to_url().to_string(), "/about");
    }

    #[wasm_bindgen_test]
    fn test_contacts() {
        let url: Url = "/contacts".parse().unwrap();
        let route = Routes::from_url(url).unwrap();
        assert_eq!(route, Routes::Contacts { show_map: None });
        assert_eq!(route.to_url().to_string(), "/contacts");
    }

    #[wasm_bindgen_test]
    fn test_contacts_with_search() {
        let url: Url = "/contacts?show-map".parse().unwrap();
        let route = Routes::from_url(url).unwrap();
        assert_eq!(
            route,
            Routes::Contacts {
                show_map: Some(true)
            }
        );
        // TODO: It seems that currently `Url` can't handle search
        // key without value.
        assert_eq!(route.to_url().to_string(), "/contacts?show-map=");
    }

    #[wasm_bindgen_test]
    fn test_manager_dashboard() {
        let url: Url = "/manager".parse().unwrap();
        let route = Routes::from_url(url).unwrap();
        assert_eq!(route, Routes::Manager(manager::Routes::Dashboard));
        assert_eq!(route.to_url().to_string(), "/manager");
    }

    #[wasm_bindgen_test]
    fn test_manager_users_list() {
        let url: Url = "/manager/users".parse().unwrap();
        let route = Routes::from_url(url).unwrap();
        assert_eq!(
            route,
            Routes::Manager(manager::Routes::Users(manager::users::Routes::List))
        );
        assert_eq!(route.to_url().to_string(), "/manager/users");
    }

    #[wasm_bindgen_test]
    fn test_manager_users_add() {
        let url: Url = "/manager/users/add".parse().unwrap();
        let route = Routes::from_url(url).unwrap();
        assert_eq!(
            route,
            Routes::Manager(manager::Routes::Users(manager::users::Routes::Add))
        );
        assert_eq!(route.to_url().to_string(), "/manager/users/add");
    }

    #[wasm_bindgen_test]
    fn test_manager_users_edit() {
        let url: Url = "/manager/users/edit/42".parse().unwrap();
        let route = Routes::from_url(url).unwrap();
        assert_eq!(
            route,
            Routes::Manager(manager::Routes::Users(
                (manager::users::Routes::Edit {
                    id: 42,
                    groups: None
                })
            ))
        );
        assert_eq!(route.to_url().to_string(), "/manager/users/edit/42");
    }

    #[wasm_bindgen_test]
    fn test_manager_users_edit_with_search() {
        let url: Url = "/manager/users/edit/42?groups=all".parse().unwrap();
        let route = Routes::from_url(url).unwrap();
        assert_eq!(
            route,
            Routes::Manager(manager::Routes::Users(
                (manager::users::Routes::Edit {
                    id: 42,
                    groups: Some("all".into()),
                })
            ))
        );
        assert_eq!(
            route.to_url().to_string(),
            "/manager/users/edit/42?groups=all"
        );
    }
}
