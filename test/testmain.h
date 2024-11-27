#pragma once

#include <QObject>

class QNetworkAccessManager;
class RePoE;

class TestHelper : public QObject {
    Q_OBJECT
public slots:
    int run(QNetworkAccessManager& network_manager, RePoE& app);
signals:
    void finished(int status);
};

int test_main();
