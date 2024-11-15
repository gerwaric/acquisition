#include "testsettings.h"

#include <QTemporaryFile>

std::unique_ptr<TestSettings> TestSettings::NewInstance(const QString& filename) {

    // Create a temporary file.
    auto tmp = std::make_unique<QTemporaryFile>();
    tmp->open();

    // If a filename was present, copy the contents.
    if (!filename.isEmpty()) {
        QFile source(filename);
        if (source.open(QIODevice::ReadWrite)) {
            tmp->write(source.readAll());
        };
    };

    // Create the test settings object and move the temporary file
    // into it so that the temporary file with be destoyed when
    // the settings object is deleted.
    TestSettings* p = new TestSettings(std::move(tmp));

    // Return a unique pointer because that's what the Application
    // object is expecting. This also ensures the object will
    // be destroyed when it goes out of scope.
    return std::unique_ptr<TestSettings>(p);
}

TestSettings::TestSettings(std::unique_ptr<QTemporaryFile> tmp) :
    QSettings(tmp->fileName(), QSettings::IniFormat),
    tmp_(std::move(tmp))
{}
