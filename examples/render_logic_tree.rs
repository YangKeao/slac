use slac::calculate::{AtomRegistry, DumpTerm};
use slac::sla::*;

use std::sync::Arc;

fn main() {
    macro_rules! ec2_infra {
        ($name: ident) => {
            let $name = Service::known_sla(stringify!($name), 0.9999);
        };
    }

    macro_rules! aws_connection {
        ($name: ident) => {
            let $name = Service::known_sla(stringify!($name), 0.9999);
        };
    }

    ec2_infra!(infra_a);
    ec2_infra!(infra_b);
    ec2_infra!(infra_c);
    ec2_infra!(infra_d);
    ec2_infra!(infra_e);

    aws_connection!(connection_a);
    aws_connection!(connection_b);
    aws_connection!(connection_c);
    aws_connection!(connection_d);

    let svc_c = Service::dependencies(vec![Dependency::Service(infra_b)]);

    let svc_d = Service::dependencies(vec![Dependency::Service(infra_c.clone())]);
    let svc_e = Service::dependencies(vec![Dependency::Service(infra_c.clone())]);
    let svc_b = Service::dependencies(vec![Dependency::Service(infra_c)]);

    let svc_g = Service::dependencies(vec![Dependency::Service(infra_e)]);

    let group_a = Group::new(vec![svc_d, svc_e, svc_c.clone(), svc_g], 2);

    let svc_a = Service::dependencies(vec![
        Dependency::Service(infra_a),
        Dependency::Service(connection_a),
        Dependency::Group(group_a),
        Dependency::Service(connection_b),
        Dependency::Service(svc_c),
    ]);

    let svc_f = Service::dependencies(vec![
        Dependency::Service(infra_d),
        Dependency::Service(connection_c),
        Dependency::Service(svc_a),
        Dependency::Service(connection_d),
        Dependency::Service(svc_b),
    ]);

    let mut atom_registry = AtomRegistry::new();
    let term = svc_f.dump_term(&mut atom_registry);
    let term_without_none = term.remove_none().unwrap();

    let mut f = std::fs::File::create("logic_tree.dot").unwrap();
    dot::render(&term, &mut f).unwrap();

    let mut f = std::fs::File::create("logic_tree_remove_none.dot").unwrap();
    dot::render(&term_without_none, &mut f).unwrap();

    println!("probability result: {}", term.calc());
}
