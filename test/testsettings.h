#pragma once

#include <QSettings>
#include <QString>

#include <memory>

class QTemporaryFile;

// Setting subclass that stores information in a temporary file
// that is automatically removed when the object is destructed.
class TestSettings : public QSettings {
public:
    static std::unique_ptr<TestSettings> NewInstance(const QString& filename = "");
private:
    explicit TestSettings(std::unique_ptr<QTemporaryFile> tmp);
    std::unique_ptr<QTemporaryFile> tmp_;
};
