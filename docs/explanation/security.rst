:relatedlinks: [Ubuntu&#32;Security&#32;reporting&#32;and&#32;disclosure&#32;policy](https://ubuntu.com/security/disclosure-policy)

.. _security_overview:

Security Overview
=================

This document provides an overview of the security aspects of Hardware API,
including potential risks and information security measures in place to protect
sensitive data.

Risks
-----

The main risk associated with Hardware API is the exposure of sensitive hardware
information that could be used to identify vulnerabilities in a system. If an
attacker gains access to detailed hardware information, they could potentially
exploit known vulnerabilities in specific hardware components or configurations.

Information Security
--------------------

The Hardware API client collects and processes hardware information of the
running system, gathered from `SMBIOS`_. The information collected by Hardware
API is sensitive, but does not include
:abbr:`PII (Personally Identifiable Information)`, user credentials, or
activity. The information collected is limited to manufacturer/vendor, model,
and versions of hardware components, firmware, kernel, OS, and BIOS. These are
listed in detail in :ref:`certification_status`.

Isolation & Containerization
----------------------------

The client is designed to minimize the amount of sensitive information
it can access. Both the snap and the deb package are confined using `AppArmor`_,
which restricts the client's access to only the necessary system resources and
files required to gather hardware information.

The ``hwctl`` `snap`_ is packaged with strict confinement and has a limited set
of interfaces required to access system information.

The ``hwctl`` deb package includes an AppArmor profile that restricts
access to only the necessary system resources and files required to gather
hardware information.

Cryptography
------------

The information is transmitted securely to the Hardware API server using
:abbr:`TLS (Transport Layer Security)`, ensuring that the data is protected
during transit.

Security Reporting and Disclosure
---------------------------------

Please refer to the `Security Policy`_ in the `canonical/hardware-api`_
repository for details on reporting security issues.

The Ubuntu `Security reporting and disclosure policy`_ contains more information
about what you can expect when you contact us and what we expect from you.
