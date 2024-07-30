#pragma once

#include <QObject>

class ItemsManager;
class BuyoutManager;
class Shop;

class TestShop : public QObject {
    Q_OBJECT
public:
    TestShop(
        ItemsManager& items_manager,
        BuyoutManager& buyout_manager,
        Shop& shop);
private slots:
    void SocketedGemsNotLinked();
    void TemplatedShopGeneration();
private:
    ItemsManager& items_manager_;
    BuyoutManager& buyout_manager_;
    Shop& shop_;
};
