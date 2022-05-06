use slac::calculate::DumpTerm;
use slac::sla::*;

fn main() {
    fn ec2_infra() -> Infra {
        Infra::new(0.99)
    }
    fn aws_connection() -> Connection {
        Connection::new(0.99)
    }

    let infra_a = ec2_infra();
    let program_a = Program::new(&infra_a);

    let infra_b = ec2_infra();
    let program_c = Program::new(&infra_b);

    let infra_c = ec2_infra();
    let program_d = Program::new(&infra_c);
    let program_e = Program::new(&infra_c);
    let program_b = Program::new(&infra_c);

    let infra_e = ec2_infra();
    let program_g = Program::new(&infra_e);

    let group_a = Group::new(2);
    unsafe {
        group_a.add(&program_d);
        group_a.add(&program_e);
        group_a.add(&program_c);
        group_a.add(&program_g);
    }

    let svc_a = Service::Internal(InternalService::new(GroupOrProgram::Group(&group_a)));
    let svc_b = Service::Internal(InternalService::new(GroupOrProgram::Program(&program_c)));

    let connection_a = aws_connection();
    let connection_b = aws_connection();
    unsafe {
        program_a.depend(&connection_a, &svc_a);
        program_a.depend(&connection_b, &svc_b);
    }

    let infra_d = ec2_infra();
    let program_f = Program::new(&infra_d);
    let svc_c = Service::Internal(InternalService::new(GroupOrProgram::Program(&program_a)));
    let svc_d = Service::Internal(InternalService::new(GroupOrProgram::Program(&program_b)));
    let connection_c = aws_connection();
    let connection_d = aws_connection();
    unsafe {
        program_f.depend(&connection_c, &svc_c);
        program_f.depend(&connection_d, &svc_d);
    }

    let end_svc = Service::Internal(InternalService::new(GroupOrProgram::Program(&program_f)));

    let term = end_svc.dump_term();
    let term_without_none = term.remove_none().unwrap();

    let mut f = std::fs::File::create("logic_tree.dot").unwrap();
    dot::render(&term, &mut f).unwrap();

    let mut f = std::fs::File::create("logic_tree_remove_none.dot").unwrap();
    dot::render(&term_without_none, &mut f).unwrap();
}
