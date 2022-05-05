# SLAC - SLA Calculator

A simple propositional logic probability calculator, with a typical senerio: calculate the theoretical SLA.

## Model

### Representation

The description model is simple and straightforward, without much abstraction, so it's expected to generate the cluster config by a script, but not write it by hand. It has five terms: `Service`, `Infra`, `Connection`, `Program`, `Group`. 

- An `Infra` is a machine with predefined SLA.
- A `Connection` is a network connection with predefined SLA. According to your consideration, it could represent a connection between two machines, or a total connection plane.
- A `Program` is a running software on a `Infra`. It could depend on some `Service` through a `Connection`.
- A `Service` is abstract. It refers to an external service (with predefined SLA), or a group, a program.
- A `Group` is abstract. It represents the HA properbility of a some program. A `Group` is considered unavailable iff the number of available program is less than the `minReplia`.

Represent your service with these five terms, the `slac` can help you to calculate the theoretical SLA, with the assumption that your custom program is 100% reliable.

### Calculation

We can represent the event "Service A is available" by the intersect or union of "X is available", where X is with predefined SLA. After the 1:1 representation, we can simplify the expression to CNF, and expand it to the direct sum and minus between intersection of events.

With the assumption that "X is available" (where X is with predefined SLA) is independent, we can calculate the final SLA.

## TODO

- [ ] Provide a easy to use cluster/application abstraction
- [ ] Calculate the credit and multiple-steps SLA
- [ ] Support per-minute SLA model
- [ ] Consider the dependency of down minutes
- [ ] Illustrate the affect of an error, and lead the user to verify this hyponsis through Chaos Mesh.
- [ ] Support dependent event.