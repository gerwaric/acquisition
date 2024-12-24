Boost Header-only Distribution
------------------------------

"Boost, but only the header files."

This repo is meant to provide a lightweight, minimal Boost distribution to
satisfy SciPy's limited set of Boost dependencies.  No binaries are features
relying on compiled libraries are available through this header-only
package.

Structure
---------

::

    project-root
    ├── boost/ : Contents of Boost includes directory (generated)
    ├── Boost_README_[version].md : Boost README file (generated)
    ├── LICENSE_1_0.txt : Boost license (generated)
    ├── make_headers.py : script to create generated files
    └── README.rst : this README file

Updating
--------

To regenerate the header files or update to a new version of Boost, the
`make_headers.py` script can be modified and run on a Linux, MacOS, or Windows
machine that has access to the internet.  It will automatically populate the
`boost/` directory with the generated header files from a minimal build.  It can
run very simply as follows:

.. code-block:: bash

    python make_headers.py -v --boost-version 1.75.0

To update the version of Boost, give a different Boost version to the
`--boost-version` option in the format: `[major].[minor].[patch]`.
The `-v` option is optional and turns on verbose logging.

Patches
-------

Patches (e.g., from cherry-picking upstream commits) can be applied by placing
`.patch` files generated using `git diff` in the `patches` directory.  See
`PATCHES <patches/PATCHES.rst>`_ for more information.
