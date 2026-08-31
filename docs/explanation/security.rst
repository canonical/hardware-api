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

The ``hwctl-daemon`` collects and processes hardware information of the
running system, gathered from `SMBIOS`_. The information collected by Hardware
API is sensitive, but does not include
:abbr:`PII (Personally Identifiable Information)`, user credentials, or
activity. The information collected is limited to manufacturer/vendor, model,
and versions of hardware components, firmware, kernel, OS, and BIOS. These are
listed in detail in :ref:`certification_status`.

Isolation & Containerization
----------------------------

The client is split into an unprivileged ``hwctl`` CLI and a privileged,
socket-activated ``hwctl-daemon``. The CLI holds no privileged access and talks
to the daemon over a local `Varlink`_ Unix socket; only the daemon collects and
submits hardware data.

The daemon is confined to minimize the amount of sensitive information it can
access. The ``hwctl`` `snap`_ is packaged with strict confinement, enforced by
`AppArmor`_, and its ``hwctl-daemon`` app exposes only the interfaces needed to
read system information (for example, ``hardware-observe``).

The daemon caches the collected hardware data and the server response on disk
at ``/var/snap/hwctl/current/hw_cache.json``, owned by root with mode ``0600``,
so it is readable only by root.

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
