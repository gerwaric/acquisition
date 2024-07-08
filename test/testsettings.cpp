#include "testsettings.h"

#include <QTemporaryFile>

std::unique_ptr<TestSettings> TestSettings::NewInstance(const QString& filename) {
    auto tmp = std::make_unique<QTemporaryFile>();
    tmp->open();
    if (!filename.isEmpty()) {
        QFile source(filename);
        if (source.open(QIODevice::ReadWrite)) {
            tmp->write(source.readAll());
        };
    };
    TestSettings* p = new TestSettings(std::move(tmp));
    return std::unique_ptr<TestSettings>(p);
}

TestSettings::TestSettings(std::unique_ptr<QTemporaryFile> tmp) :
    QSettings(tmp->fileName(), QSettings::IniFormat),
    tmp_(std::move(tmp))
{}
