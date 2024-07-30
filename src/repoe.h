#pragma once

#include <QObject>

class QNetworkAccessManager;
class QString;

enum class ProgramState;

class RePoE : public QObject {
    Q_OBJECT
public:
    RePoE(QObject* parent, QNetworkAccessManager& network_manager);
    void Init();
    bool IsInitialized() const { return initialized_; };
signals:
    void StatusUpdate(ProgramState state, const QString& status);
    void finished();
public slots:
    void OnItemClassesReceived();
    void OnBaseItemsReceived();
    void OnStatTranslationReceived();
private:
    void GetStatTranslations();
    QNetworkAccessManager& network_manager_;
    static bool initialized_;
    static QStringList GetTranslationUrls();
};
