"""Town coordinates and ordered West Coast itineraries used by the offline map build."""

# City centers, longitude/latitude. OSRM snaps these to the drivable road network.
CITIES = {
    'Seattle': (-122.3321, 47.6062, 'WA'),
    'Portland': (-122.6765, 45.5231, 'OR'),
    'San Francisco': (-122.4194, 37.7749, 'CA'),
    'Los Angeles': (-118.2437, 34.0522, 'CA'),
    'Sacramento': (-121.4944, 38.5816, 'CA'),
    'San Diego': (-117.1611, 32.7157, 'CA'),
    'Spokane': (-117.4260, 47.6588, 'WA'),
    'Missoula': (-113.9940, 46.8721, 'MT'),
    'Billings': (-108.5007, 45.7833, 'MT'),
    'Rapid City': (-103.2310, 44.0805, 'SD'),
    'Sioux Falls': (-96.7311, 43.5446, 'SD'),
    'Boise': (-116.2023, 43.6150, 'ID'),
    'Salt Lake City': (-111.8910, 40.7608, 'UT'),
    'Reno': (-119.8138, 39.5296, 'NV'),
    'Cheyenne': (-104.8202, 41.1400, 'WY'),
    'Las Vegas': (-115.1398, 36.1699, 'NV'),
    'Flagstaff': (-111.6513, 35.1983, 'AZ'),
    'Albuquerque': (-106.6504, 35.0844, 'NM'),
    'Amarillo': (-101.8313, 35.2220, 'TX'),
    'Phoenix': (-112.0740, 33.4484, 'AZ'),
    'Tucson': (-110.9747, 32.2226, 'AZ'),
    'El Paso': (-106.4850, 31.7619, 'TX'),
    'San Antonio': (-98.4936, 29.4241, 'TX'),
    'Denver': (-104.9903, 39.7392, 'CO'),
    'North Platte': (-100.7654, 41.1403, 'NE'),
    'Omaha': (-95.9345, 41.2565, 'NE'),
    'Des Moines': (-93.6250, 41.5868, 'IA'),
    'Iowa City': (-91.5302, 41.6611, 'IA'),
    'Minneapolis': (-93.2650, 44.9778, 'MN'),
    'La Crosse': (-91.2396, 43.8014, 'WI'),
    'Madison': (-89.4012, 43.0731, 'WI'),
    'Kansas City': (-94.5786, 39.0997, 'MO'),
    'Columbia': (-92.3341, 38.9517, 'MO'),
    'St. Louis': (-90.1994, 38.6270, 'MO'),
    'Springfield, IL': (-89.6501, 39.7817, 'IL'),
    'Austin': (-97.7431, 30.2672, 'TX'),
    'Waco': (-97.1467, 31.5493, 'TX'),
    'Dallas': (-96.7970, 32.7767, 'TX'),
    'Oklahoma City': (-97.5164, 35.4676, 'OK'),
    'Tulsa': (-95.9928, 36.1540, 'OK'),
    'Joplin': (-94.5133, 37.0842, 'MO'),
    'Springfield, MO': (-93.2923, 37.2089, 'MO'),
    'Chicago': (-87.6298, 41.8781, 'IL'),
    'South Bend': (-86.2520, 41.6764, 'IN'),
    'Toledo': (-83.5552, 41.6639, 'OH'),
    'Cleveland': (-81.6944, 41.4993, 'OH'),
    'Pittsburgh': (-79.9959, 40.4406, 'PA'),
    'Cumberland': (-78.7625, 39.6529, 'MD'),
    'Hagerstown': (-77.7200, 39.6418, 'MD'),
    'Frederick': (-77.4105, 39.4143, 'MD'),
    'D.C.': (-77.0091, 38.8899, 'DC'),
}

COMMON = ['Chicago', 'South Bend', 'Toledo', 'Cleveland', 'Pittsburgh',
          'Cumberland', 'Hagerstown', 'Frederick', 'D.C.']
PLAINS = ['Denver', 'North Platte', 'Omaha', 'Des Moines', 'Iowa City']
MISSOURI = ['Kansas City', 'Columbia', 'St. Louis', 'Springfield, IL']
STARTS = {
    'journalist': ['Seattle', 'Spokane', 'Missoula', 'Billings', 'Rapid City',
                   'Sioux Falls', 'Minneapolis', 'La Crosse', 'Madison'],
    'organizer': ['Portland', 'Boise', 'Salt Lake City'] + PLAINS,
    'whistleblower': ['San Francisco', 'Reno', 'Salt Lake City'] + PLAINS,
    'lobbyist': ['Los Angeles', 'Las Vegas', 'Flagstaff', 'Albuquerque',
                 'Amarillo', 'Oklahoma City'] + MISSOURI,
    'staffer': ['Sacramento', 'Reno', 'Salt Lake City', 'Cheyenne'] + PLAINS,
    'satirist': ['San Diego', 'Phoenix', 'Tucson', 'El Paso', 'San Antonio',
                 'Austin', 'Waco', 'Dallas', 'Oklahoma City', 'Tulsa', 'Joplin',
                 'Springfield, MO', 'St. Louis', 'Springfield, IL'],
}


def setting(name, eastern_leg):
    """The landscape follows the outbound leg, rather than the simulation day."""
    state = CITIES[name][2]
    if name in ('Frederick', 'D.C.'):
        return 'Beltway', 'open-beltway-parkway' if name == 'D.C.' else 'open-beltway-suburbs'
    if eastern_leg:
        scene = 'open-appalachian-ridge' if name in ('Pittsburgh', 'Cumberland', 'Hagerstown') else 'open-rustbelt-lakeside' if name in ('Toledo', 'Cleveland') else 'open-rustbelt-foundry'
        return 'RustBelt', scene
    if state in ('WA', 'OR', 'CA'):
        return 'PacificCoast', 'open-pacific-northwest' if state in ('WA', 'OR') else 'open-california-hills'
    if name in ('Las Vegas', 'Flagstaff', 'Albuquerque', 'Phoenix', 'Tucson', 'El Paso'):
        return 'Southwest', 'open-southwest-desert'
    if state in ('ID', 'MT', 'WY', 'UT', 'NV', 'CO'):
        scene = 'open-great-basin' if name in ('Reno', 'Boise') else 'open-mountain-west'
        # Denver and Billings head out of the mountains into open plains.
        if name in ('Denver', 'Billings', 'Cheyenne'):
            scene = 'open-heartland-prairie'
        return 'MountainWest', scene
    return 'Heartland', 'open-heartland-orchard' if name in ('Des Moines', 'Iowa City', 'La Crosse', 'Madison') else 'open-heartland-prairie'
