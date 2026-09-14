($tabs[0]|map({(.id):.name})|add) as $tn
| map(select([(.json.explicitMods//[])[]|.description]|any(test("Reservation Efficiency")))
      |{id:.id[0:8],league,tab:$tn[.location_id],mod:([(.json.explicitMods//[])[]|.description|select(test("Reservation"))][0])})
