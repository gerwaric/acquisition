#pragma once

#include <QObject>
#include <QString>

class BuyoutHelperPrivate;

class BuyoutHelper : public QObject {
    Q_OBJECT
public:
    BuyoutHelper();
    ~BuyoutHelper();
    void validate(const QString& filename);
private:
    std::unique_ptr<BuyoutHelperPrivate> m_helper;
};
