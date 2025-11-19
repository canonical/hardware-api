Get started with ``hwctl``
==========================

This tutorial shows you how to use the ``hwctl`` command-line tool to check
the certification status of your hardware configuration.

Install ``hwctl``
-----------------

On Questing Quokka (25.10) or later, you can install ``hwctl`` using ``apt``:

.. code:: bash

   sudo apt-get install hwctl

On earlier Ubuntu releases, you can install the ``hwctl`` `snap`_:

.. code:: bash

   sudo snap install hwctl

Check certification status
--------------------------

.. note::

    The client requires root access since we collect the hardware information using SMBIOS data.

To check your machine's certification status, simply run ``hwctl`` with root privileges:

.. terminal::

   sudo hwctl

   {
     "status": "Not Seen"
   }

In the above example, the specific configuration of the machine has not been
certified yet, so the status is "Not Seen". Depending on your hardware
configuration, you may see different certification statuses, such as
"Certified", "Not Seen" (not certified), "Certified Image Exists", or
"Related Certified System Exists".

Next steps
----------

Now that you know how to use ``hwctl``, you can learn more about the API response
types and data structures in the `hwapi schema`_.
