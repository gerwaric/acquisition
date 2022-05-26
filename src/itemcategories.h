#ifndef ITEMCATEGORIES_H
#define ITEMCATEGORIES_H

#include <string>
#include <map>

extern std::map<std::string, std::string> itemClassKeyToValue;
extern std::map<std::string, std::string> itemClassValueToKey;
extern std::map<std::string, std::string> itemBaseType_NameToClass;

class ItemCategories
{
public:
	ItemCategories();
};

#endif // ITEMCATEGORIES_H
