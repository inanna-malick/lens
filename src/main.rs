mod functor;
use std::borrow::Cow;

use functor::*;
mod traversal;
use prism::{idx, PrismExt};
use traversal::*;
mod lens;
use lens::*;
mod json;
use json::*;
mod util;
use util::*;

mod prism;

mod example;
use example::*;

use serde_json::{json, Map, Value};

// todo revisit later

fn main() {
    let str = json!({"imagine": "nothing"});
    let obj = json!({"imagine": { "something": {"else": "entirely"}}});
    let obj_arr = json!({"imagine": { "something": [{"else": "entirely"}, null, {"woah": "yeah"}]}});

    let prism = object_key("imagine").and_if(object_key("something"));
    let prism2 = object_key("imagine").and_if(object_key("something")).and_if(JArray).and_if(idx(2));

    println!(
        "prism tovec: s vo: ${:?}, o vs: ${:?}, obj_arr {:?}",
        prism.t_to_vec(str.clone()),
        prism.t_to_vec(obj.clone()),
        prism.t_to_vec(obj_arr.clone())
    );

    println!(
        "prism2 tovec: s vo: ${:?}, o vs: ${:?}, obj_arr {:?}",
        prism2.t_to_vec(str.clone()),
        prism2.t_to_vec(obj.clone()),
        prism2.t_to_vec(obj_arr.clone())
    );

    println!(
        "prism to_opt: s vo: ${:?}, o vs: ${:?}, obj_arr {:?}",
        prism.t_to_opt(str.clone()),
        prism.t_to_opt(obj.clone()),
        prism.t_to_vec(obj_arr.clone())
    );

    let addkey = |mut v: Value| {
        if let Some(m) = v.as_object_mut() {
            m.insert("new".to_string(), Value::Null);
        }
        v
    };

    println!(
        "addkey over prism2: s vo: ${:?}, o vs: ${:?}, obj_arr {:?}",
        prism2.over(str.clone(), addkey),
        prism2.over(obj.clone(), addkey),
        prism2.over(obj_arr.clone(), addkey)
    );


    println!(
        "addkey over prism: s vo: ${:?}, o vs: ${:?}, obj_arr {:?}",
        prism.over(str, addkey),
        prism.over(obj, addkey),
        prism.over(obj_arr, addkey)
    );

    let point = Point { x: 1, y: 1 };

    let atom = Atom {
        name: "helium".to_string(),
        point,
    };

    println!("atom: {:?}", atom);
    println!("atom point: {:?}", Atom::point().getter(atom.clone()));
    println!(
        "atom x: {:?}",
        Atom::point().and(Point::x()).getter(atom.clone())
    );

    println!(
        "a, b, {:?}, {:?}",
        atom.clone().test1(),
        atom.clone().test2()
    );

    let shifted = Atom::point().and(Point::x()).over(atom, |x| x + 1);

    println!("shifted atom: {:?}", shifted);

    let water = Molecule {
        name: "water".to_string(),
        atoms: vec![
            Atom {
                name: "hydrogen".to_string(),
                point: Point { x: 0, y: 0 },
            },
            Atom {
                name: "hydrogen".to_string(),
                point: Point { x: 1, y: 1 },
            },
            Atom {
                name: "oxygen".to_string(),
                point: Point { x: 2, y: 2 },
            },
        ],
    };

    println!("water: {:?}", water);

    let molecule_x_coords = Molecule::atoms()
        .all(elems())
        .and(Atom::point())
        .and(Point::x());

    let shifted = molecule_x_coords.over(water, |x| x + 1);
    println!("shifted water: {:?}", shifted);

    let x_coords = molecule_x_coords.t_to_vec(shifted);
    println!("shifted water x coords: {:?}", x_coords);
}
