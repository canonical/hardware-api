Get started with ``hwctl``
==========================

This tutorial shows you how to use the ``hwctl`` command-line tool to check
the certification status of your hardware configuration.

Install ``hwctl``
-----------------

``hwctl`` is provided as a snap. Install it on any supported Ubuntu release
with:

.. code:: bash

   sudo snap install hwctl

Check certification status
--------------------------

To check your machine's certification status, run ``hwctl``. It connects to
the ``hwctl-daemon``, which collects the hardware information and talks to the
server.

.. terminal::
   :copy:
   :input: hwctl

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
