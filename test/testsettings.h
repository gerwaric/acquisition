#pragma once

#include <QSettings>
#include <QString>

#include <memory>

class QTemporaryFile;

class TestSettings : public QSettings {
public:
    static std::unique_ptr<TestSettings> NewInstance(const QString& filename = "");
private:
    explicit TestSettings(std::unique_ptr<QTemporaryFile> tmp);
    std::unique_ptr<QTemporaryFile> tmp_;
};
