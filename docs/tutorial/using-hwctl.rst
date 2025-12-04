Get started with ``hwctl``
==========================

This tutorial shows you how to use the ``hwctl`` command-line tool to check
the certification status of your hardware configuration.

Install ``hwctl``
-----------------

.. tab-set::
   .. tab-item:: Deb (Ubuntu)
      :sync: deb

      On Questing Quokka (25.10) or later, you can install ``hwctl`` using
      ``apt``:

      .. code:: bash

         sudo apt-get install hwctl

   .. tab-item:: Snap
      :sync: snap

      You can install the ``hwctl`` `snap`_ on any supported Ubuntu release:

      .. code:: bash

         sudo snap install hwctl

Check certification status
--------------------------

.. note::

    The client requires root access since we collect the hardware information
    using `SMBIOS`_ data.

To check your machine's certification status, simply run ``hwctl`` with root privileges:

.. terminal::
   :copy:
   :input: sudo hwctl

   {
     "status": "Not Seen"
   }

In the above example, the specific configuration of the machine has not been
certified yet, so the status is "Not Seen". Depending on your hardware
configuration, you may see different certification statuses. For more
information on the possible statuses, refer to :ref:`certification_status`.

Next steps
----------

Now that you know how to use ``hwctl``, you can learn more about the API response
types and data structures in the :ref:`Hardware API OpenAPI schema <openapi>`.
