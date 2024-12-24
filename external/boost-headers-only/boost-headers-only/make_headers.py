"""Generate header files from Boost source distribution."""

import argparse
import gzip
import logging
import pkg_resources
import pathlib
import platform
import urllib.request
import shutil
import subprocess
import tarfile
import tempfile
from time import time
logging.basicConfig()


def _generate_headers(ver: str, verbose: bool):
    # setup logging
    logger = logging.getLogger('boost-make-headers')
    if verbose:
        logger.setLevel(logging.INFO)

    # base is where this script is located
    base = pathlib.Path(__file__).parent
    BOOST_VER_DOT = pkg_resources.parse_version(ver).base_version
    BOOST_VER_UND = '_'.join(BOOST_VER_DOT.split('.'))

    # download gzipped tarball
    # location of tarballs can also be found at https://www.boost.org/users/download/
    archive_name = f'boost_{BOOST_VER_UND}'
    url = f'https://boostorg.jfrog.io/artifactory/main/release/{BOOST_VER_DOT}/source/{archive_name}.tar.gz'
    t0 = time()
    with urllib.request.urlopen(url) as response:
        logger.info(f'Starting download of BOOST {BOOST_VER_DOT} source distribution')
        with gzip.GzipFile(fileobj=response) as uncompressed, tempfile.NamedTemporaryFile(delete=False, suffix='.tar') as ntf:
            logger.info(f'Saving Boost tarball to {ntf.name}')
            shutil.copyfileobj(uncompressed, ntf)
            logger.info(f'Finished downloading and uncompressing in {time() - t0:.2f} seconds')
            ntf.flush()
    logger.info('Starting to extract')
    t0 = time()
    try:
        with tarfile.open(ntf.name, 'r') as tar, tempfile.TemporaryDirectory() as tmpdir:
            dst = pathlib.Path(tmpdir)
            tar.extractall(path=dst)
            logger.info(f'Finished extracting to {dst / archive_name} in {time() - t0:.2f} seconds')

            # configure build by calling bootstrap.sh
            logger.info('Starting configure')
            t0 = time()
            bootstrap_script = './bootstrap.sh' if platform.system() != 'Windows' else 'boostrap.bat'
            with open(base / 'bootstrap.log', 'w') as fp:
                if subprocess.run([f'{bootstrap_script}',
                                   f'--prefix={dst / "boost_tmp_build"}',
                                   '--with-libraries=math'],
                                  cwd=dst / archive_name,
                                  stdout=fp, stderr=fp).returncode != 0:
                    raise ValueError(f'Failed to run {bootstrap_script}!')
            logger.info(f'Completed configure in {time() - t0:.2f} seconds')

            # Do the build, will create some binaries but we will ignore these
            logger.info('Starting build')
            t0 = time()
            with open(base / 'b2.log', 'w') as fp:
                b2_script = './b2' if platform.system() != 'Windows' else 'b2'
                if subprocess.run([b2_script, 'install'],
                                  cwd=dst / archive_name,
                                  stdout=fp, stderr=fp).returncode != 0:
                    raise ValueError(f'Failed to run {b2_script} install!')

                # ensure the include/ directory really does exist where we think it does
                if not (dst / 'boost_tmp_build').exists():
                    raise ValueError('Headers failed to generate!')
            logger.info(f'Completed build in {time() - t0:.2f} seconds')

            # Remove old headers and replace with new headers and move License, README
            logger.info('Updating header files')
            if (base / 'boost').exists():
                shutil.rmtree(base / 'boost')
                for f in pathlib.Path(__file__).parent.glob("Boost_*_README.md"):
                    f.unlink()
            shutil.move(dst / 'boost_tmp_build/include/boost', base / 'boost')
            shutil.move(dst / archive_name / 'LICENSE_1_0.txt', base / 'LICENSE_1_0.txt')
            shutil.move(dst / archive_name / 'README.md', base / f'Boost_{BOOST_VER_UND}_README.md')
    finally:
        # We want to save the tar file as a temporary file and simultaneously
        # extract it, meaning it will need to be opened in a context manager
        # multiple times.  While Linux can handle nested context managers
        # using the same file handle, Windows cannot.  So we have to mark the
        # temporary file "ntf" as delete=False, close its context manager, and
        # then ensure cleanup happens in this "finally" statement
        ntf.close()

    logger.info('Done creating base Boost headers!')

    logger.info("Applying patches...")
    patch_dir = pathlib.Path(__file__).parent / "patches"
    for f in patch_dir.glob("*.patch"):
        if subprocess.run(["git", "apply", str(f)], cwd=base).returncode != 0:
            logger.error(f"Failed to apply patch: {f}.  Skipping.")
        logger.info(f"Applied {f}")
    else:
        logger.info("Found no patches to apply!")

    logger.info("Done!")


if __name__ == '__main__':
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument('--boost-version', type=str,
                        help='Boost version to download formatted as [major].[minor].[patch].', required=True)
    parser.add_argument('-v', action='store_true', help='Enable verbose logging.', default=False)
    args = parser.parse_args()
    _generate_headers(ver=args.boost_version, verbose=args.v)
