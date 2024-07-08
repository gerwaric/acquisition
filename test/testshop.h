#pragma once

#include <QObject>

class Application;

class TestShop : public QObject {
    Q_OBJECT
public:
    TestShop();
private slots:
    void initTestCase();
    void SocketedGemsNotLinked();
    void TemplatedShopGeneration();
private:
    std::unique_ptr<Application> app_;
};
