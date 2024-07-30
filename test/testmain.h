#pragma once

#include <QObject>

class Application;

class TestHelper : public QObject {
    Q_OBJECT
public slots:
    int run(Application& app);
signals:
    void finished(int status);
};


int test_main();
