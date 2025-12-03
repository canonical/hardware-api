.. _certification_status:

Certification Status
=====================

This document provides an overview of the different certification statuses that
Hardware API assigns based on the collected system information.

To learn more about the Hardware Certification process, please refer to the
`Hardware Certification documentation`_. To browse certified devices, visit the
`Ubuntu Certified`_ page.

Information Collected
---------------------

Hardware API collects various pieces of information about the system to
determine the :term:`platform` and :term:`configuration` of the device because
Ubuntu Certified status is granted at these broader abstractions of hardware.
In order to determine the platform and configuration of a system, Hardware API
collects the following information:

.. tab-set::
    .. tab-item:: AMD64
        :sync: amd64

        - :abbr:`OS (Operating System)` information.

          - OS release type & version. See :manpage:`os-release(5)`.
          - Running kernel (name, version, signature, and loaded modules). See :manpage:`lsmod(8)`.
          - Image type (if available)

        - :abbr:`DMI (Desktop Management Interface)` table hardware system
          information. See `SMBIOS`_.

          - BIOS identity (name, version, and release date)
          - Motherboard identity (manufacturer, product name, and version)
          - Chassis identity (chassis type, manufacturer, sku, and version)
          - CPU identity (identifier, frequency, manufacturer, and version)
          - PCI IDs of peripherals attached
          - Amount and type of memory installed
          - Amount and type of non-volatile storage installed

    .. tab-item:: ARM64
        :sync: arm64

        .. note::
            Hardware API does not yet support ARM64. This information is based
            on a work-in-progress implementation and may change in the future.

        - :abbr:`SoC (system-on-chip)` identity information (``/sys/devices/soc*``)
        - Device trees
    
To learn more about the risks and information security measures in place to
protect sensitive data, please refer to :ref:`security_overview`.

Certification Statuses
----------------------

Based on the collected information, Hardware API assigns one of the following
certification statuses to the system:

- **Not Seen**: Hardware API was unable to uniquely identify the system's
  platform or configuration as part of the certification program. This response
  is typically returned for systems that are not yet certified or for which
  insufficient information could be collected.
- **Certified**: Hardware API was able to uniquely identify the system's
  platform and configuration. A matching certified configuration/platform exists
  in the database and the machine is running the OS that the
  configuration/platform is certified to work on.
- **Related Certified System Exists**: Hardware API was able to identify the
  system's platform. The system's configuration is either not certified or could
  not be uniquely identified. However, there exists at least one certified
  configuration within the same platform.
- **Certified Image Exists**: Hardware API was able to uniquely identify the
  system's platform and configuration. However, the configuration is not
  certified for the running OS image type. There exists at least one OS image
  type for which the specific configuration is certified.

To learn more about the specific API responses and data formats, please refer to
the Hardware API :ref:`openapi` schema.
