// Copyright 2022 Chaos Mesh Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use crate::calculate::{Atom, Term, DumpTerm};

use std::{cell::RefCell, sync::Arc};

use itertools::Itertools;

#[derive(Debug)]
pub struct Infra {
    sla: f64
}

impl Infra {
    pub fn new(sla: f64) -> Infra {
        Infra {
            sla
        }
    }
}

impl Atom for &Infra {
    fn probability(&self) -> f64 {
        1.0 - self.sla
    }
    fn name(&self) -> String {
        format!("Infra{:#x}", *self as *const _ as usize)
    }
}

#[derive(Debug)]
pub struct Connection {
    sla: f64
}

impl Connection {
    pub fn new(sla: f64) -> Connection {
        Connection {
            sla
        }
    }
}

impl Atom for &Connection {
    fn probability(&self) -> f64 {
        1.0 - self.sla
    }
    fn name(&self) -> String {
        format!("Conn{:#x}", *self as *const _ as usize)
    }
}

pub struct Program<'a> {
    infra: &'a Infra,
    depends: RefCell<Vec<(&'a Connection, &'a dyn IService)>>,
}

impl<'a> Program<'a> {
    pub fn new(infra: &'a Infra) -> Self {
        Self { infra, depends: RefCell::new(Vec::new()) }
    }

    pub unsafe fn depend<'c: 'a, 's1: 'a, 's2: 'a>(&self, connection: &'c Connection, svc: &'s1 dyn IService) {
        self.depends.borrow_mut().push((connection, svc));
    }
}

pub trait IProgram: DumpTerm {

}

impl<'a> IProgram for Program<'a> {

}

impl<'a> DumpTerm for Program<'a> {
    fn dump_term(&self) -> Term {
        let mut term = Term::Atom(Arc::new(self.infra));
        for (conn, svc) in self.depends.borrow().iter() {
            let both_broke = Term::Union { lhs: Box::new(Term::Atom(Arc::new(*conn))), rhs: Box::new(svc.dump_term()) };

            term = Term::Union { lhs: Box::new(term), rhs: Box::new(both_broke) };
        }

        term
    }
}

#[derive(Debug)]
pub struct ExternalService {
    sla: f64
}

impl ExternalService {
    pub fn new(sla: f64) -> ExternalService {
        ExternalService { sla }
    }
}

impl Atom for &ExternalService {
    fn probability(&self) -> f64 {
        self.sla
    }
    fn name(&self) -> String {
        format!("Svc{:#x}", *self as *const _ as usize)
    }
}

pub struct InternalService<'a> {
    internal: GroupOrProgram<'a>
}

impl<'a> InternalService<'a> {
    pub fn new(internal: GroupOrProgram<'a>) -> Self {
        Self {
            internal,
        }
    }
}

pub enum Service<'a> {
    External(ExternalService),
    Internal(InternalService<'a>)
}

pub trait IService: DumpTerm {
}

impl<'a> IService for Service<'a> {
}

impl<'a> DumpTerm for Service<'a> {
    fn dump_term(&self) -> Term {
        match self {
            Service::External(svc) => Term::Atom(Arc::new(svc)),
            Service::Internal(internal) => {
                match internal.internal {
                    GroupOrProgram::Group(group) => group.dump_term(),
                    GroupOrProgram::Program(program) => program.dump_term(),
                }
            }
        }
    }
}

pub struct Group<'a> {
    programs: RefCell<Vec<&'a dyn IProgram>>,
    min_replica: usize,
}

impl<'a> Group<'a> {
    pub fn new(min_replica: usize) -> Self {
        Self {
            programs: RefCell::new(Vec::new()),
            min_replica,
        }
    }

    pub unsafe fn add(&self, program: &'a dyn IProgram) {
        self.programs.borrow_mut().push(program);
    }
}

impl<'a> DumpTerm for Group<'a> {
    fn dump_term(&self) -> Term {
        // TODO: do some optimization here
        // e.g. if there are some isomorphic relationships between programs, we can calculate the 
        // probability in toltal
        let mut term = Term::None;

        let total = self.programs.borrow().len();
        for fail_count in total - self.min_replica + 1..total {
            for combination in self.programs.borrow().iter().combinations(fail_count) {
                let mut combination_term = Term::None;
                for program in combination {
                    combination_term = Term::Intersect { lhs: Box::new(combination_term), rhs: Box::new(program.dump_term()) };
                }
    
                term = Term::Union { lhs: Box::new(term), rhs: Box::new(combination_term) };
            }
        }

        term
    }
}

pub enum GroupOrProgram<'a> {
    Group(&'a Group<'a>),
    Program(&'a Program<'a>)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_define() {
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
        // Then the end_svc is what we need to calculate
        println!("{:?}", end_svc.dump_term());
    }
}
