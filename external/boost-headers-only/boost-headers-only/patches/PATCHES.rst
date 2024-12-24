Applying Patch Files
--------------------

All `.patch` files found in this directory will be applied to this repository.
Any that fail will be ignored with a logging error message.

To generate patch file, the following command can be used:

::

    git diff [commit] [commit] > patches/myCoolPatchFile.patch

Any variation on the `git diff` command should also succeed in producing a
suitable patch file.
