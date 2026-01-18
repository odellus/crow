# openhands.sdk.security

> API reference for openhands.sdk.security module

### class SecurityRisk

Bases: `str`, `Enum`

Security risk levels for actions.

Based on OpenHands security risk levels but adapted for agent-sdk.
Integer values allow for easy comparison and ordering.

#### Properties

* `description`: str
  Get a human-readable description of the risk level.
* `visualize`: Text
  Return Rich Text representation of this risk level.

#### Methods

#### HIGH = 'HIGH'

#### LOW = 'LOW'

#### MEDIUM = 'MEDIUM'

#### UNKNOWN = 'UNKNOWN'

#### get\_color()

Get the color for displaying this risk level in Rich text.

#### is\_riskier()

Check if this risk level is riskier than another.

Risk levels follow the natural ordering: LOW is less risky than MEDIUM, which is
less risky than HIGH. UNKNOWN is not comparable to any other level.

To make this act like a standard well-ordered domain, we reflexively consider
risk levels to be riskier than themselves. That is:

for risk\_level in list(SecurityRisk):
: assert risk\_level.is\_riskier(risk\_level)

# More concretely:

assert SecurityRisk.HIGH.is\_riskier(SecurityRisk.HIGH)
assert SecurityRisk.MEDIUM.is\_riskier(SecurityRisk.MEDIUM)
assert SecurityRisk.LOW\.is\_riskier(SecurityRisk.LOW)

This can be disabled by setting the reflexive parameter to False.

* Parameters:
  other ([SecurityRisk\*](#class-securityrisk)) – The other risk level to compare against.
  reflexive (bool\*) – Whether the relationship is reflexive.
* Raises:
  `ValueError` – If either risk level is UNKNOWN.


---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://docs.openhands.dev/llms.txt