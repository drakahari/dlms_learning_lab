# Review network diagnoses and proposed configuration

These two CCNA v1.1 activities address parts of 6.4 and 6.6. They follow
the staged change and controller policy exercises. Each presents an
annotated example, an independent guided workspace and a different
assignment.

## Independent checks of supplied diagnostic output

The source supplies a fictional generated cause, a traffic forecast,
earlier readings and a later reading. The learner runs the incident
probe against the supplied starting state and compares observed carrier,
desired policy and installed rules. A later reading becomes visible
only when requested. The forecast is judged by the alert decision at
one threshold on this sample. A match on one sample says nothing about
future accuracy.

The repair publishes an existing virtual rule and exercises the affected
exchange and distinct control cases. A speculative carrier adjustment
does not repair a stale rule. No learner prompt is sent to a model.
Training, prediction generation, data collection, actual network analysis
and any real repair action are outside the simulation.

## Desired configuration plan

The course distinguishes Terraform's proposed resource plan from
Ansible check and diff mode. Check results depend on module support.
The virtual task reads the lab's own bounded JSON profile and binding
format. It checks an ordinary current/desired resource difference and
exposes Create, Update and Remove rows. A preview has no side effects.
Applying a current, valid plan changes only the virtual resource
table. Invalid input leaves the previous applied state intact.

The work order requires preserving the supplied unused recovery profile,
retiring a separate obsolete record, and leaving unrelated ports and
profiles intact. Actual simulated Ethernet paths verify the intended
result. After applying, a new comparison establishes that the draft
and its current resources agree. Several valid profile assignments can
satisfy the same paths when they respect the full authorized scope.

No Terraform or Ansible tool, parser, provider, module, deployment,
credentials or external network is used.

Primary references checked November 17, 2026:
[Cisco CCNA v1.1](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf) ·
[Ansible check and diff mode](https://docs.ansible.com/projects/ansible/latest/playbook_guide/playbooks_checkmode.html) ·
[Terraform plan](https://developer.hashicorp.com/terraform/cli/commands/plan)
