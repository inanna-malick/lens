mod functor;
use functor::*;
mod traversal;
use traversal::*;
mod lens;
use lens::*;
mod json;
use json::*;
mod util;
use util::*;

mod example;
use example::*;

use serde_json::{json, Map, Value};

// todo revisit later

fn main() {
    let str = Value::String("test".to_string());

    let obj = json!({"key": { "something": "else"}});

    let jst = ValueObject()
        .all(ObjectKey {
            key: "key".to_string(),
        })
        .all(ValueObject());

    println!(
        "tovec: s vo: ${:?}, o vs: ${:?}",
        jst.t_to_vec(str.clone()),
        jst.t_to_vec(obj.clone())
    );

    let addkey = |mut m: Map<String, Value>| {
        m.insert("new".to_string(), Value::Null);
        m
    };

    println!(
        "addkey over: s vo: ${:?}, o vs: ${:?}",
        jst.over(str, addkey),
        jst.over(obj, addkey)
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
